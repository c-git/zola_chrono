use clap::Parser;
use cli::Cli;
use log::{info, LevelFilter};
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};
mod cli;

fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();
    init_logging(cli.log_level.into())?;
    info!("Cli: {cli:#?}");
    run(&cli)
}

fn init_logging(log_level: LevelFilter) -> anyhow::Result<()> {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{h({d} {l} {t} - {m})}{n}")))
        .build();
    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(log_level))?;
    log4rs::init_config(config)?;
    Ok(())
}

fn run(cli: &Cli) -> anyhow::Result<()> {
    todo!()
}
