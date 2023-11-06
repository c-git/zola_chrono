use clap::Parser;
use log::debug;
use zola_page_date_setter::{self, cli::Cli, init_logging, run};
fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();
    init_logging(cli.log_level.into())?;
    debug!("Cli: {cli:#?}");
    run(&cli)
}
