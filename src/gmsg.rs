// gmsg.rs
use crate::ai::GenerateCommitMsg;
use crate::config::LoadedConfig;
use crate::tui::{editor::Editor, selector::Selector};
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

    #[arg(short = 'a', long = "amend", num_args = 0..=1)]
    pub amend: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(name = "config.provider")]
    ConfigProvider,

    #[command(name = "config.models")]
    ConfigModel,
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
                let mut terminal = ratatui::init();
                if let Some(selected) = Selector::new(providers).run(&mut terminal)? {
                    config.write_provider(selected)?;
                }
                ratatui::restore();
            }
            Command::ConfigModel => {
                let models = config.list_models().await?;
                let mut terminal = ratatui::init();
                if let Some(selected) = Selector::new(models).run(&mut terminal)? {
                    config.write_model(selected)?;
                }
                ratatui::restore();
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

        let diff = crate::git::get_diff(&repository)?;
        let agent = crate::ai::build_commit_agent(
            config.config.ai.provider.clone(),
            config.config.ai.model.clone(),
            config.config.ai.prompt.as_deref(),
        )
        .context("Could not bootstrap agent")?;
        let mut out = Self::strip_backtick(&agent.generate_commit_msg(&diff).await?);

        if self.interactive || self.amend.is_some() {
            let mut terminal = ratatui::init();
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
}