use std::path::PathBuf;

use clap::Parser;
#[derive(Parser)]
#[command(version,long_about=None)]
pub struct Gmsg {
    #[arg(short, long, value_name = "PATH")]
    pub path: Option<PathBuf>,
    #[arg(short, long = "dry-run")]
    dryrun: bool,
}
