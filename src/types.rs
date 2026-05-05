use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about = "Generate conventional commit messages")]
pub struct Gmsg {
    /// Path to the git repository
    #[arg(short, long, value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// This opens an editor for you to modify commit messages before saving
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,

    /// This copies the generated message to your clipboard and exits
    #[arg(short = 'c', long = "copy")]
    pub copy: bool,
}
