use anyhow::bail;
use clap::Parser;
use tracing::{debug, error};
use tracing_subscriber::{fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter};
use zola_chrono::{self, run, Cli};

fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();
    init_tracing();
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

fn init_tracing() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();
}
