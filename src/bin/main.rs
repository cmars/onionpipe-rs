use std::fs::File;
use std::io::Read;

use clap::{Parser, Subcommand};

use onionpipe::{config, OnionPipe, PipeError, Result};

#[derive(Parser)]
#[command(name = "onionpipe")]
#[command(bin_name = "onionpipe")]
struct Cli {
    #[arg(long)]
    config: Option<std::path::PathBuf>,

    #[clap(subcommand)]
    commands: Option<Commands>,

    forwards: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(subcommand)]
    Service(ServiceCommands),
}

#[derive(Subcommand)]
enum ServiceCommands {
    Add { name: String },
    Delete { name: String },
    List,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match &cli.commands {
        Some(Commands::Service(ServiceCommands::Add { ref name })) => add_service(name).await,
        Some(Commands::Service(ServiceCommands::Delete { ref name })) => delete_service(name).await,
        Some(Commands::Service(ServiceCommands::List)) => list_services().await,
        None => run(cli).await,
    };
    let rc = match result {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    };
    std::process::exit(rc)
}

async fn add_service(name: &str) -> Result<()> {
    let config_dir = match dirs::config_dir() {
        Some(dir) => dir,
        None => {
            return Err(PipeError::CLI("failed to locate config dir".to_string()));
        }
    };
    let secrets_dir = config_dir.join("onionpipe");
    let mut secret_store = onionpipe::secrets::SecretStore::new(secrets_dir.to_str().unwrap());
    let key_bytes = secret_store.ensure_service(name)?;
    let onion_addr = torut::onion::TorSecretKeyV3::from(key_bytes)
        .public()
        .get_onion_address()
        .to_string();
    println!("{}\t{}", name, onion_addr);
    Ok(())
}

async fn delete_service(name: &str) -> Result<()> {
    let config_dir = match dirs::config_dir() {
        Some(dir) => dir,
        None => {
            return Err(PipeError::CLI("failed to locate config dir".to_string()));
        }
    };
    let secrets_dir = config_dir.join("onionpipe");
    let mut secret_store = onionpipe::secrets::SecretStore::new(secrets_dir.to_str().unwrap());
    match secret_store.delete_service(name)? {
        Some(()) => {
            println!("service {} deleted", name);
            Ok(())
        }
        None => Err(PipeError::CLI(
            format!("{}: service not found", name).to_string(),
        )),
    }
}

async fn list_services() -> Result<()> {
    let config_dir = match dirs::config_dir() {
        Some(dir) => dir,
        None => {
            return Err(PipeError::CLI("failed to locate config dir".to_string()));
        }
    };
    let secrets_dir = config_dir.join("onionpipe");
    let secret_store = onionpipe::secrets::SecretStore::new(secrets_dir.to_str().unwrap());
    let services = secret_store.list_services()?;
    for service_name in services {
        let key_bytes = secret_store.get_service(&service_name)?.unwrap();
        let onion_addr = torut::onion::TorSecretKeyV3::from(key_bytes)
            .public()
            .get_onion_address()
            .to_string();
        println!("{}\t{}", service_name, onion_addr);
    }
    Ok(())
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
