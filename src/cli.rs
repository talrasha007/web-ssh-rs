use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(about = "Minimal wetty-style web SSH terminal")]
pub struct Cli {
    /// Path to the SSH private key used to authenticate to every target host.
    #[arg(long)]
    pub identity_file: PathBuf,

    /// Passphrase for the identity file, if it is encrypted.
    #[arg(long)]
    pub key_passphrase: Option<String>,

    /// Address the HTTP server listens on.
    #[arg(long, default_value = "0.0.0.0:8080")]
    pub bind_addr: SocketAddr,
}
