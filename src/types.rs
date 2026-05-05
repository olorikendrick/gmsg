use std::path::PathBuf;

use clap::Parser;
#[derive(Parser)]
#[command(version,long_about=None)]
pub struct Gmsg {
    #[arg(short, long, value_name = "PATH")]
    pub path: Option<PathBuf>,
    #[arg(short = 'd', long = "dry-run")]
    pub dryrun: bool,
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,
}
