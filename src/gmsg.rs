use crate::ai::CompletionClient;
use crate::config::Config;
use crate::git::{commit, get_diff};
use crate::tui::{TerminalGuard, editor::Editor, selector::Selector};
use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use git2::Repository;
use std::io::IsTerminal;
use std::path::PathBuf;
use strum::IntoEnumIterator;

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
    Config(ConfigArgs),
}

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigSubcommand,
}

#[derive(Subcommand)]
pub enum ConfigSubcommand {
    Provider,
    Models,
    Prompt { prompt: String },
    Show,
}

impl Gmsg {
    pub async fn run() -> anyhow::Result<()> {
        let app = Self::parse();
        let wdir = app.working_dir()?;
        let mut config = Config::load(wdir.clone())?;

        if let Some(command) = &app.command {
            return app.handle_command(command, &mut config).await;
        }

        app.handle_commit(&config, wdir).await
    }

    async fn handle_command(&self, command: &Command, config: &mut Config) -> anyhow::Result<()> {
        match command {
            Command::Config(args) => match &args.action {
                ConfigSubcommand::Provider => {
                    let providers = crate::ai::Provider::iter().collect::<Vec<_>>();
                    let mut terminal = TerminalGuard::new();
                    if let Some(selected) = Selector::new(providers).run(&mut terminal)? {
                        config.ai.provider = selected.clone();
                        let models = selected.initialize()?.list_models().await?;
                        if let Some(model) = Selector::new(models).run(&mut terminal)? {
                            config.ai.model = model;
                        }
                        config.save_to(None)?;
                    }
                }
                ConfigSubcommand::Models => {
                    let models = config
                        .ai
                        .provider
                        .clone()
                        .initialize()?
                        .list_models()
                        .await?;
                    let mut terminal = TerminalGuard::new();
                    if let Some(model) = Selector::new(models).run(&mut terminal)? {
                        config.ai.model = model;
                        config.save_to(None)?;
                    }
                }
                ConfigSubcommand::Prompt { prompt } => {
                    config.ai.sys_prompt = prompt.clone();
                    config.save_to(None)?;
                }
                ConfigSubcommand::Show => {
                    println!("{:#?}", config);
                }
            },
        }
        Ok(())
    }

    async fn handle_commit(&self, config: &Config, wdir: PathBuf) -> anyhow::Result<()> {
        let repository = Repository::discover(&wdir).context(
            "Failed to open a git repository. Check if it exists or if you have necessary permissions",
        )?;

        let diff = get_diff(&repository)?;

        if !self.amend && diff.is_none() {
            eprintln!("No staged changes detected");
            return Ok(());
        }

        let provider = config.ai.provider.initialize()?;
        let agent = provider
            .create_completion_client(config.ai.model.clone(), config.ai.sys_prompt.clone())?;

        if self.amend {
            self.make_amends(&repository, diff, agent.as_ref()).await?;
            return Ok(());
        }

        let diff = diff.expect("Diff should not be none");
        self.dispatch(&repository, diff, agent.as_ref()).await
    }

    async fn dispatch(
        &self,
        repository: &Repository,
        diff: String,
        agent: &dyn CompletionClient,
    ) -> anyhow::Result<()> {
        let (raw_msg, _usage) = agent.generate_commit_msg(&diff).await?;
        let mut msg = Self::strip_backtick(&raw_msg);

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

        OutputAction::new(self, msg).execute(repository)
    }

    async fn make_amends(
        &self,
        repository: &Repository,
        diff: Option<String>,
        agent: &dyn CompletionClient,
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
                let (msg, _) = agent
                    .generate_commit_msg(&format!(
                        "Amend this commit message: {}\n\nWith this new diff:\n{}",
                        prev_msg, diff
                    ))
                    .await?;
                msg
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
        index.read(true).context("Failed to read index")?;
        let tree_oid = index.write_tree()?;
        let tree = repository.find_tree(tree_oid)?;

        prev_commit.amend(Some("HEAD"), None, None, None, Some(&out), Some(&tree))?;

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
                #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
                {
                    use arboard::Clipboard;
                    let mut clipboard =
                        Clipboard::new().context("Failed to get system clipboard")?;
                    clipboard
                        .set_text(&msg)
                        .context("Failed to set clipboard")?;
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    eprintln!("Copied to clipboard: {}", &msg);
                }
                #[cfg(target_os = "android")]
                {
                    let status = std::process::Command::new("termux-clipboard-set")
                        .args([&msg])
                        .status();

                    match status {
                        Ok(s) if s.success() => eprintln!("Copied to Android clipboard: {}", &msg),
                        _ => {
                            eprintln!("Failed to copy to Android clipboard.");
                            eprintln!(
                                "Hint: Make sure you have installed the Termux:API add-on app and run `pkg install termux-api`."
                            );
                        }
                    }
                }

                #[cfg(not(any(
                    target_os = "linux",
                    target_os = "windows",
                    target_os = "macos",
                    target_os = "android"
                )))]
                {
                    eprintln!("Copying is not yet supported on this platform.");
                }
            }
            OutputAction::Commit(msg) => match commit(repository, &msg) {
                Ok(_) => eprintln!("Committed with message:\n{}", msg),
                Err(e) => eprintln!("Error while committing: {:?}", e),
            },
            OutputAction::Pipe(msg) => {
                println!("{}", msg);
            }
        }
        Ok(())
    }

    fn new(cli: &Gmsg, msg: String) -> Self {
        if cli.copy {
            Self::Copy(msg)
        } else if !std::io::stdout().is_terminal() {
            Self::Pipe(msg)
        } else {
            Self::Commit(msg)
        }
    }
}
