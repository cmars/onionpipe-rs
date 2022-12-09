use std::str::FromStr;
use std::{cell, env, fs, io, net, path, rc, result};

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
    #[error("socks error")]
    Socks(#[from] tokio_socks::Error),
    #[error("join error")]
    Join(#[from] tokio::task::JoinError),
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
        let socks_sock = data_dir.join("socks.sock").to_str().unwrap().into();
        Ok(OnionPipe {
            temp_dir: Some(temp_dir),
            data_dir: data_dir.to_str().unwrap().into(),
            control_sock: control_sock,
            socks_sock: socks_sock,
            exports: self.exports,
            imports: self.imports,
        })
    }
}

pub struct OnionPipe {
    temp_dir: Option<tempfile::TempDir>,
    data_dir: String,
    control_sock: String,
    socks_sock: String,
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

async fn on_event_noop(
    _: torut::control::AsyncEvent<'static>,
) -> result::Result<(), torut::control::ConnError> {
    Ok(())
}

const IMPORT_BUF_LEN: usize = 65536;

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
        ac.set_async_event_handler(Some(on_event_noop));
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

        self.forward_imports().await?;

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

    async fn forward_imports(&self) -> Result<()> {
        for i in 0..self.imports.len() {
            let import = &self.imports[i];
            let import_addr = format!("{}:{}", import.remote_addr, import.remote_port);
            let socks_addr = self.socks_sock.to_string();

            tokio::spawn(run_import(
                import.local_addr.to_string(),
                socks_addr,
                import_addr.to_string(),
            ));
        }
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
                format!(
                    "SocksPort unix:{} OnionTrafficOnly",
                    self.socks_sock.as_str()
                )
                .into(),
            ))
            .start_background();
    }
}

async fn run_import(local_addr: String, socks_addr: String, import_addr: String) -> Result<()> {
    let local_listener = tokio::net::TcpListener::bind(local_addr).await?;
    loop {
        let (local_stream, _) = local_listener.accept().await?;
        println!("got connection");
        let proxy_stream = tokio::net::UnixStream::connect(socks_addr.as_str()).await?;
        let remote_stream =
            tokio_socks::tcp::Socks5Stream::connect_with_socket(proxy_stream, import_addr.as_str())
                .await?;

        tokio::spawn(forward_stream(local_stream, remote_stream));
    }
}

async fn forward_stream(
    mut local: tokio::net::TcpStream,
    mut remote: tokio_socks::tcp::Socks5Stream<tokio::net::UnixStream>,
) -> Result<()> {
    let (mut local_read, mut local_write) = local.split();
    let (mut remote_read, mut remote_write) = remote.split();
    tokio::select! {
        _ = async {
            tokio::io::copy(&mut remote_read, &mut local_write).await?;
            Ok::<_, PipeError>(())
        } => {}
        _ = async {
            tokio::io::copy(&mut local_read, &mut remote_write).await?;
            Ok::<_, PipeError>(())
        } => {}
        else => {}
    };
    Ok(())
}

#[tokio::main]
async fn main() {
    // TODO: config / cli parser
    let pbaddr = torut::onion::OnionAddressV3::from_str(
        "piratebayo3klnzokct3wt5yyxb2vpebbuyjl7m623iaxmqhsd52coid",
    )
    .unwrap();
    let mut onion_pipe = OnionPipe::defaults()
        .export(Export {
            local_addr: net::SocketAddr::new(
                net::IpAddr::from(net::Ipv4Addr::new(127, 0, 0, 1)),
                8000,
            ),
            remote_port: 8000,
            remote_key: None,
        })
        .import(Import {
            remote_addr: torut::onion::OnionAddress::V3(pbaddr),
            remote_port: 80,
            local_addr: net::SocketAddr::new(
                net::IpAddr::from(net::Ipv4Addr::new(127, 0, 0, 1)),
                3000,
            ),
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
