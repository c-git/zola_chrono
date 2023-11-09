use anyhow::bail;
use clap::Parser;
use log::{debug, error};
use zola_chrono::{self, init_logging, run, Cli};
fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();
    init_logging(cli.log_level.into())?;
    debug!("Cli: {cli:#?}");
    let stats = run(&cli)?;
    println!("File Stats: {stats}");
    if stats.errors() == 0 {
        if cli.should_check_only && stats.changed() > 0 {
            println!("{} files would have been changed", stats.changed());
            std::process::exit(2);
        }
        Ok(())
    } else {
        let msg = format!("Run FAILED! {} errors", stats.errors());
        error!("{msg}");
        bail!("{msg}");
    }
}
