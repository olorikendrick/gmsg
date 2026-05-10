use crate::ai::GenerateCommitMsg;
// gmsg.rs
use crate::config::LoadedConfig;
use crate::git::get_diff;
use crate::tui::{editor::Editor, selector::Selector,TerminalGuard};
use anyhow::Context;
use arboard::Clipboard;
use clap::{Parser, Subcommand};
use git2::Repository;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;


#[derive(Parser)]
#[command(version, about = "Generate conventional commit messages")]
pub struct Gmsg {
    #[arg(short, long, value_name = "PATH")]
    pub path: Option<PathBuf>,

    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,

    #[arg(short = 'c', long = "copy")]
    pub copy: bool,

    #[arg(short = 'a', long = "amend")]
    pub amend: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(name = "config.provider")]
    ConfigProvider,

    #[command(name = "config.models")]
    ConfigModel,
    #[command(name = "config.prompt")]
    Prompt { prompt: String },
}

impl Gmsg {
    pub async fn run() -> anyhow::Result<()> {
        let cli = Self::parse();

        if let Some(command) = &cli.command {
            return cli.handle_command(command).await;
        }

        cli.handle_commit().await
    }

    async fn handle_command(&self, command: &Command) -> anyhow::Result<()> {
        let wdir = self.working_dir()?;
        let mut config = LoadedConfig::load(&wdir)?;

        match command {
            Command::ConfigProvider => {
                let providers = LoadedConfig::list_providers();
                let mut terminal = TerminalGuard::new();
                if let Some(selected) = Selector::new(providers).run(&mut terminal)? {
                    config.write_provider(selected)?;
                }
                let models = config.list_models().await?;

                if let Some(selected) = Selector::new(models).run(&mut terminal)? {
                    config.write_model(selected)?;
                }
                ratatui::restore();
            }
            Command::ConfigModel => {
                let models = config.list_models().await?;
                let mut terminal = TerminalGuard::new();
                if let Some(selected) = Selector::new(models).run(&mut terminal)? {
                    config.write_model(selected)?;
                }
                ratatui::restore();
            }
            Command::Prompt { prompt } => {
                config.write_prompt(prompt.to_owned())?;
            }
        }
        Ok(())
    }

    async fn handle_commit(&self) -> anyhow::Result<()> {
        let wdir = self.working_dir()?;
        let config = LoadedConfig::load(&wdir)?;

        let repository = Repository::discover(&wdir).context(
            "Failed to open a git repository. Check if it exists or if you have necessary permissions",
        )?;

        let diff_result = get_diff(&repository)?;

        let agent = crate::ai::build_commit_agent(
            config.config.ai.provider.clone(),
            config.config.ai.model.clone(),
            config.config.ai.prompt.as_deref(),
        )
        .context("Could not bootstrap agent")?;

        if self.amend {
            Self::make_amends(&repository, diff_result.as_ref(), &agent).await?;
            return Ok(());
        }
        let Some(diff) = diff_result else {
            eprintln!("No Staged Changes Detected");
            return Ok(());
        };

        let mut out = Self::strip_backtick(&agent.generate_commit_msg(&diff).await?);

        if self.interactive {
            let mut terminal = TerminalGuard::new();
            out = Editor::from(out)
                .run(&mut terminal)
                .context("Failed to initialize inline editor")?;
            ratatui::restore();

            if out.is_empty() {
                eprintln!("Aborted commit operation");
                return Ok(());
            }
        }

        if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
            println!("{}", out);
            return Ok(());
        }

        if self.copy {
            let mut clipboard = Clipboard::new().context("Failed to get system clipboard")?;
            clipboard
                .set_text(&out)
                .context("Failed to set clipboard")?;
            thread::sleep(Duration::from_secs(2));
            eprintln!("Copied to clipboard: {}", out);
            return Ok(());
        }

        match crate::git::commit(&repository, &out) {
            Ok(_) => eprintln!("Committed with message:\n{}", out),
            Err(e) => eprintln!("Error while committing: {:?}", e),
        }

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
        diff: Option<&String>,
        agent: &Box<dyn GenerateCommitMsg>,
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
        ratatui::restore();

        if out.is_empty() {
            eprintln!("Aborted amend operation");
            return Ok(());
        }
let mut index = repository.index()?;
index.read(true).context("Failed to read index")?;  // force read from disk
let tree_oid = index.write_tree()?;
let tree = repository.find_tree(tree_oid)?;

prev_commit.amend(Some("HEAD"), None, None, None, Some(&out), Some(&tree))?;

        Ok(())
    }
}
