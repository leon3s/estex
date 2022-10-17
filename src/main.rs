use clap::Parser;

mod cli;
mod error;
mod event;
mod server;
mod routes;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    let args = cli::Args::parse();
    let ehandler = event::init().await;
    server::start(&args.ip_address, &args.port, ehandler).await?;
    Ok(())
}
