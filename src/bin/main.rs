use std::fs::File;
use std::io::Read;

use clap::Parser;

use onionpipe::{config, OnionPipe, Result};

#[derive(Parser)]
#[command(name = "onionpipe")]
#[command(bin_name = "onionpipe")]
struct Cli {
    #[arg(long)]
    config: Option<std::path::PathBuf>,

    forwards: Vec<String>,
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
    unsafe {
        libc::umask(0o077);
    }

    let mut pipe_builder = OnionPipe::defaults();

    if let Some(config_dir) = dirs::config_dir() {
        let secrets_dir = config_dir.join("onionpipe");
        pipe_builder = pipe_builder.secrets_dir(secrets_dir.to_str().unwrap());
    }

    let cfg: config::Config;
    if let Some(config_path) = cli.config.as_ref() {
        let mut config_file = File::open(config_path)?;
        let mut config_json = String::new();
        config_file.read_to_string(&mut config_json)?;
        drop(config_file);

        cfg = serde_json::from_str(&config_json)?;
    } else {
        cfg = cli.forwards.try_into()?;
    }

    pipe_builder = pipe_builder.config(cfg)?;

    let mut onion_pipe = pipe_builder.new().await?;
    onion_pipe.run().await?;
    Ok(())
}
