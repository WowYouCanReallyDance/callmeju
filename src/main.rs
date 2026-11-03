use anyhow::Result;
use clap::Parser;

use callmeju::config::{Cli, Config};
use callmeju::session::Session;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();
    let config = Config::load(&cli)?;
    tracing::info!(backend = %config.server.base_url, "callmeju starting");

    let session = Session::new(config)?;
    tokio::select! {
        res = session.run() => res,
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("ctrl-c received, shutting down");
            Ok(())
        }
    }
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}
