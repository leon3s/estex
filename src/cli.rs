use clap::Parser;

/// ntrft
/// Ntex Raft Http2 server example
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
   /// Ip address to bind the webserver to
   #[arg(short, long, default_value = "0.0.0.0")]
   pub(crate) ip_address: String,

   /// Port to bind
   #[arg(short, long, default_value = "80")]
   pub(crate) port: i32,
}
