use anyhow::bail;
use clap::Parser;
use log::debug;
use zola_chrono::{self, cli::Cli, init_logging, run};
fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();
    init_logging(cli.log_level.into())?;
    debug!("Cli: {cli:#?}");
    let stats = run(&cli)?;
    println!("File Stats: {stats}");
    if stats.errors() == 0 {
        Ok(())
    } else {
        bail!("Got {} errors", stats.errors());
    }
}
