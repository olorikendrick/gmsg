use crate::ai::{CompletionClient, ModelEntry, ModelProvider, Provider};
use crate::config::{AiConfig, Config};
use crate::git::{commit, get_diff};
use crate::tui::{TerminalGuard, editor::Editor, selector::Selector};
use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
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
    /// Manage configuration settings
    Config(ConfigArgs),
}

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigSubcommand,
}

#[derive(Subcommand)]
pub enum ConfigSubcommand {
    /// Select AI provider and model
    Provider,

    /// Select AI model from provider
    Models,

    /// Pass in a custom prompt for the model
    Prompt {
        /// The custom prompt string
        prompt: String,
    },

    /// Print current configuration
    Show,
}
impl Gmsg {
    pub async fn run() -> anyhow::Result<()> {
        let app = Gmsg::parse();
        let wdir = app.path.unwrap_or(std::env::current_dir()?);
        let config = Config::load(wdir);
        dbg!(config);

        Ok(())
    }
}
