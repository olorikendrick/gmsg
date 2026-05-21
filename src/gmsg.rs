use crate::ai::GenerateCommitMsg;
// gmsg.rs
use crate::config::Config;
use crate::git::{commit, get_diff};
use crate::tui::{TerminalGuard, editor::Editor, selector::Selector};
use anyhow::{Context, Result};
use arboard::Clipboard;
use clap::{Parser, Subcommand};
use git2::Repository;
use std::io::IsTerminal;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about = "Generate conventional commit messages")]
pub struct Gmsg {
    /// Path to repository
    #[arg(short, long, value_name = "PATH")]
    pub path: Option<PathBuf>,
    /// Edit commits in an editor
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,
    /// Copy generated to clipboard and exit
    #[arg(short = 'c', long = "copy")]
    pub copy: bool,
    /// Amend previous commit
    #[arg(short = 'a', long = "amend")]
    pub amend: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Select AI provider and model
    #[command(name = "config.provider")]
    ConfigProvider,
    ///Select AI model from provider
    #[command(name = "config.models")]
    ConfigModel,
    /// Pass in a custom prompt for the model
    #[command(name = "config.prompt")]
    Prompt { prompt: String },
    ///print config
    #[command(name = "config.show")]
    ConfigShow,
}

impl Gmsg {
    pub async fn run() -> anyhow::Result<()> {
        let cli = Self::parse();
        let wdir = cli.working_dir()?;

        let mut config = Config::load(&wdir)?;
        eprintln!("{:?}", &config);
        if let Some(command) = &cli.command {
            return cli.handle_command(command, &mut config).await;
        }

        cli.handle_commit(&config, wdir).await
    }

    async fn handle_command(&self, command: &Command, config: &mut Config) -> anyhow::Result<()> {
        match command {
            Command::ConfigProvider => {
                let providers = Config::list_providers();
                let mut terminal = TerminalGuard::new();
                if let Some(selected) = Selector::new(providers).run(&mut terminal)? {
                    config.write_provider(selected.clone())?;

                    let models = Config::list_models(config.ai.provider.clone()).await?;

                    if let Some(selected) = Selector::new(models).run(&mut terminal)? {
                        config.write_model(selected)?;
                    }
                }
            }
            Command::ConfigModel => {
                let provider = config.ai.provider.clone();
                let models = Config::list_models(provider).await?;
                let mut terminal = TerminalGuard::new();
                if let Some(selected) = Selector::new(models).run(&mut terminal)? {
                    config.write_model(selected)?;
                }
            }
            Command::Prompt { prompt } => {
                config.write_prompt(prompt.to_owned())?;
            }
            Command::ConfigShow => {}
        }
        Ok(())
    }

    async fn handle_commit(&self, config: &Config, wdir: PathBuf) -> anyhow::Result<()> {
        let repository = Repository::discover(&wdir).context(
            "Failed to open a git repository. Check if it exists or if you have necessary permissions",
        )?;

        let diff = get_diff(&repository)?;

        if !self.amend && diff.is_none() {
            eprintln!("No Staged change detected");

            return Ok(());
        }

        let agent = crate::ai::build_commit_agent(
            config.ai.provider.clone(),
            config.ai.model.clone(),
            config.ai.prompt.as_deref(),
        )
        .context("Could not bootstrap agent")?;
        if self.amend {
            Self::make_amends(&repository, diff, agent.as_ref()).await?;
            return Ok(());
        }
        let diff = diff.expect("Diff should not be none");
        self.dispatch(&repository, diff, agent.as_ref()).await?;

        Ok(())
    }

    async fn dispatch(
        &self,
        repository: &Repository,
        diff: String,
        agent: &dyn GenerateCommitMsg,
    ) -> anyhow::Result<()> {
        let mut msg = Self::strip_backtick(&agent.generate_commit_msg(&diff).await?);

        if self.interactive {
            let mut terminal = TerminalGuard::new();
            msg = Editor::from(msg)
                .run(&mut terminal)
                .context("Failed to initialize inline editor")?;

            if msg.is_empty() {
                eprintln!("Aborted commit operation");
                return Ok(());
            }
        }
        let action = OutputAction::new(self, msg);

        action.execute(repository)?;
        Ok(())
    }

    fn working_dir(&self) -> anyhow::Result<PathBuf> {
        if let Some(path) = &self.path {
            Ok(path.to_owned())
        } else {
            std::env::current_dir().context("Failed to get current working directory")
        }
    }

    fn strip_backtick(input: &str) -> String {
        input.replace('`', "")
    }

    async fn make_amends(
        repository: &Repository,
        diff: Option<String>,
        agent: &dyn GenerateCommitMsg,
    ) -> anyhow::Result<()> {
        let prev_commit = repository
            .head()
            .context("Failed to get HEAD")?
            .peel_to_commit()
            .context("Failed to peel to commit")?;

        let prev_msg = prev_commit.message().unwrap_or("").to_string();

        let editor_input = match diff {
            None => prev_msg.clone(),
            Some(diff) => {
                agent
                    .generate_commit_msg(&format!(
                        "Amend this commit message: {}\n\nWith this new diff:\n{}",
                        prev_msg, diff
                    ))
                    .await?
            }
        };

        let mut terminal = TerminalGuard::new();
        let out = Editor::from(editor_input)
            .run(&mut terminal)
            .context("Failed to initialize inline editor")?;

        if out.is_empty() {
            eprintln!("Aborted amend operation");
            return Ok(());
        }
        let mut index = repository.index()?;
        index.read(true).context("Failed to read index")?; // force read from disk
        let tree_oid = index.write_tree()?;
        let tree = repository.find_tree(tree_oid)?;

        prev_commit.amend(Some("HEAD"), None, None, None, Some(&out), Some(&tree))?;

        Ok(())
    }
}
#[derive(Debug)]
enum OutputAction {
    Copy(String),
    Commit(String),
    Pipe(String),
}

impl OutputAction {
    fn execute(self, repository: &Repository) -> Result<()> {
        match self {
            OutputAction::Copy(msg) => {
                let mut clipboard = Clipboard::new().context("Failed to get system clipboard")?;
                clipboard
                    .set_text(&msg)
                    .context("Failed to set clipboard")?;
                std::thread::sleep(std::time::Duration::from_secs(3));
                eprintln!("Copied to clipboard: {}", &msg);
            }
            OutputAction::Commit(msg) => match commit(repository, &msg) {
                Ok(_) => eprintln!("Committed with message:\n{}", msg),
                Err(e) => eprintln!("Error while committing: {:?}", e),
            },
            OutputAction::Pipe(msg) => {
                println!("{}", msg);
            }
        };
        Ok(())
    }

    fn new(cli: &Gmsg, msg: String) -> Self {
        if cli.copy {
            Self::Copy(msg)
        } else {
            let stdout = std::io::stdout();
            if !stdout.is_terminal() {
                return Self::Pipe(msg);
            }
            Self::Commit(msg)
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::ai::{MOCK_RESPONSE, build_commit_agent};
    use crate::config::Config;
    use crate::git::stage_files;
    use crate::test_utils::*;
    use arboard::Clipboard;
    use anyhow::Result;
    #[tokio::test]
    async fn test_c_flag_works() -> Result<()> {
        let (repo, dir) = setup()?;
        let path = dir.path();
        stage_files(&["test.txt".to_string()], &repo)?;
        let gmsg = Gmsg {
            path: Some(path.to_path_buf()),
            interactive: false,
            amend: false,
            copy: true,
            command: None,
        };
        let wdir = gmsg.working_dir()?;
        let mut config = Config::load(&wdir)?;
      
        gmsg.handle_commit(&config, wdir).await?;
                std::thread::sleep(std::time::Duration::from_secs(5));

        let  mut clipboard=Clipboard::new()?;
        let received=clipboard.get_text()?;


        assert_eq!(MOCK_RESPONSE,received);

        Ok(())
    }
}
