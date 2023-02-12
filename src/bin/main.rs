use std::fs::File;
use std::io::Read;

use clap::Parser;

use onionpipe::{Config, OnionPipe, Result};

#[derive(Parser)]
#[command(name = "onionpipe")]
#[command(bin_name = "onionpipe")]
struct Cli {
    #[arg(long)]
    config: std::path::PathBuf,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let rc = match run(cli).await {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    };
    std::process::exit(rc)
}

async fn run(cli: Cli) -> Result<()> {
    let mut config_file = File::open(cli.config)?;
    let mut config_json = String::new();
    config_file.read_to_string(&mut config_json)?;
    drop(config_file);

    let config: Config = serde_json::from_str(&config_json)?;
    let mut onion_pipe = OnionPipe::defaults().config(config)?.new().await?;
    onion_pipe.run().await?;
    Ok(())
}
