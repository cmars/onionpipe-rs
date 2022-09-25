use std::{env, fs, io, net, path, result};

use std::os::unix::fs::PermissionsExt;

use torut::{control, onion};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PipeError {
    #[error("timeout connecting to tor control socket")]
    ConnTimeout,
    #[error("failed to connect to tor control socket")]
    Conn(#[from] control::ConnError),
    #[error("i/o error")]
    IO(#[from] io::Error),
}

type Result<T> = result::Result<T, PipeError>;

pub struct OnionPipeBuilder {
    temp_path: path::PathBuf,
    exports: Vec<Export>,
    imports: Vec<Import>,
}

impl OnionPipeBuilder {
    pub fn temp_path(mut self, temp_path: &str) -> OnionPipeBuilder {
        self.temp_path = path::PathBuf::from(temp_path);
        self
    }

    pub fn export(mut self, export: Export) -> OnionPipeBuilder {
        self.exports.push(export);
        self
    }

    pub fn import(mut self, import: Import) -> OnionPipeBuilder {
        self.imports.push(import);
        self
    }

    pub async fn new(self) -> Result<OnionPipe> {
        let temp_dir = tempfile::tempdir_in(self.temp_path)?;
        let data_dir = temp_dir.path().join("data");
        tokio::fs::create_dir(data_dir.as_path()).await?;
        tokio::fs::set_permissions(data_dir.as_path(), fs::Permissions::from_mode(0o700)).await?;
        let control_sock = data_dir.join("control.sock").to_str().unwrap().into();
        Ok(OnionPipe {
            temp_dir: Some(temp_dir),
            data_dir: data_dir.to_str().unwrap().into(),
            control_sock: control_sock,
            exports: self.exports,
            imports: self.imports,
        })
    }
}

pub struct OnionPipe {
    temp_dir: Option<tempfile::TempDir>,
    data_dir: String,
    control_sock: String,
    exports: Vec<Export>,
    imports: Vec<Import>,
}

pub struct Export {
    local_addr: net::SocketAddr,
    remote_key: Option<onion::TorSecretKeyV3>,
    remote_port: u16,
}

pub struct Import {
    remote_addr: onion::OnionAddress,
    remote_port: u16,
    local_addr: net::SocketAddr,
}

impl OnionPipe {
    pub fn defaults() -> OnionPipeBuilder {
        OnionPipeBuilder {
            temp_path: env::temp_dir(),
            exports: vec![],
            imports: vec![],
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.start_tor();

        wait_for_file(&self.control_sock).await?;
        let s = tokio::net::UnixStream::connect(&self.control_sock).await?;
        let mut utc = control::UnauthenticatedConn::new(s);
        utc.authenticate(&control::TorAuthData::Null).await?;
        let mut ac = utc.into_authenticated().await;
        ac.set_async_event_handler(Some(|ev| async move {
            println!("event received: {:?}", ev);
            Ok(())
        }));
        ac.take_ownership().await?;

        let mut active_onions = vec![];
        for i in 0..self.exports.len() {
            let export = &self.exports[i];
            let ephemeral_key = match export.remote_key {
                Some(ref _remote_key) => None,
                None => Some(onion::TorSecretKeyV3::generate()),
            };
            let remote_key = match ephemeral_key.as_ref() {
                Some(key) => key,
                None => export.remote_key.as_ref().unwrap(),
            };
            println!(
                "forward {} => {}:{}",
                export.local_addr,
                remote_key.public().get_onion_address(),
                export.remote_port,
            );
            ac.add_onion_v3(
                remote_key,
                false,
                false,
                false,
                None,
                &mut [(export.remote_port, export.local_addr)].iter(),
            )
            .await?;
            active_onions.push(
                remote_key
                    .public()
                    .get_onion_address()
                    .get_address_without_dot_onion(),
            );
        }

        tokio::signal::ctrl_c().await?;
        println!("interrupt received, shutting down");

        for i in 0..active_onions.len() {
            match ac.del_onion(active_onions[i].as_str()).await {
                Err(control::ConnError::IOError(io_err)) => {
                    if io_err.kind() == std::io::ErrorKind::ConnectionReset {
                        // Control connection may be lost here
                        break;
                    }
                    println!("failed to delete onion: {:?}", io_err);
                }
                Err(err) => {
                    println!("failed to delete onion: {:?}", err);
                }
                _ => {}
            }
        }
        // TODO: poll w/timeout for a connection reset, ping w/ GETINFO

        // Close connection
        drop(ac);
        // Delete data dir
        tokio::fs::remove_dir_all(&self.data_dir).await?;
        // Clean up temp dir
        self.temp_dir.take().unwrap().close()?;
        Ok(())
    }

    fn start_tor(&self) -> () {
        // TODO(long-term): replace with Arti when it supports onions!
        libtor::Tor::new()
            .flag(libtor::TorFlag::ControlSocket(
                self.control_sock.as_str().into(),
            ))
            .flag(libtor::TorFlag::DataDirectory(
                self.data_dir.as_str().into(),
            ))
            // TODO: configurable log level
            .flag(libtor::TorFlag::LogTo(
                libtor::log::LogLevel::Warn,
                libtor::log::LogDestination::Stderr,
            ))
            // TODO: SocksPort unix:/path/to/socks.sock ...
            .flag(libtor::TorFlag::Custom(
                "SocksPort auto OnionTrafficOnly".into(),
            ))
            .start_background();
    }
}

#[tokio::main]
async fn main() {
    // TODO: config / cli parser
    let mut onion_pipe = OnionPipe::defaults()
        .export(Export {
            local_addr: net::SocketAddr::new(
                net::IpAddr::from(net::Ipv4Addr::new(127, 0, 0, 1)),
                8000,
            ),
            remote_port: 8000,
            remote_key: None,
        })
        .new()
        .await
        .unwrap();
    onion_pipe.run().await.unwrap();
}

async fn wait_for_file(path: &str) -> Result<()> {
    for i in 0..10 {
        match tokio::fs::metadata(path).await {
            Ok(_) => {
                return Ok(());
            }
            Err(_) => {}
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(i)).await;
    }
    Err(PipeError::ConnTimeout)
}
