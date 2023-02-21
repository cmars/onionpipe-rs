use std::fs::File;
use std::io::Read;

use clap::Parser;

use onionpipe::{config, parse, OnionPipe, Result};

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
    let mut pipe_builder = OnionPipe::defaults();
    if let Some(config_path) = cli.config.as_ref() {
        let mut config_file = File::open(config_path)?;
        let mut config_json = String::new();
        config_file.read_to_string(&mut config_json)?;
        drop(config_file);

        let cfg: config::Config = serde_json::from_str(&config_json)?;
        pipe_builder = pipe_builder.config(cfg)?;
    }

    for forward_arg in cli.forwards {
        let parsed_forward = forward_arg.parse::<parse::Forward>()?;
        let forward: config::Forward = parsed_forward.into();
        match forward {
            config::Forward::Import(import) => {
                pipe_builder = pipe_builder.import(import.try_into()?);
            }
            config::Forward::Export(export) => {
                pipe_builder = pipe_builder.export(export.try_into()?);
            }
        }
    }

    let mut onion_pipe = pipe_builder.new().await?;
    onion_pipe.run().await?;
    Ok(())
}
