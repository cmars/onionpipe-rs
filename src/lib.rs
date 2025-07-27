use std::str::FromStr;
use std::{env, fs, io, net, path, result};

use regex::Regex;
use std::os::unix::fs::PermissionsExt;
use thiserror::Error;
use torut::{control, onion};

pub mod config;
pub mod parse;
pub mod secrets;

#[derive(Error, Debug)]
pub enum PipeError {
    #[error("timeout connecting to tor control socket")]
    ConnTimeout,
    #[error("failed to connect to tor control socket")]
    Conn(#[from] control::ConnError),
    #[error("i/o error: {0}", .source)]
    IO {
        #[from]
        source: io::Error,
        //backtrace: std::backtrace::Backtrace,
    },
    #[error("socks error: {0}")]
    Socks(#[from] tokio_socks::Error),
    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("invalid socket address: {0}")]
    ParseAddr(#[from] net::AddrParseError),
    #[error("command failed: {0}")]
    CLI(String),
    #[error("invalid config: {0}")]
    Config(String),
    #[error("config parse error: {0}")]
    ConfigParse(#[from] serde_json::Error),
    #[error("secret store error: {0}")]
    SecretStore(#[from] secrets::SecretsError),
    #[error("forward parse error: {0}")]
    ForwardParse(#[from] parse::ParseError),
    #[error("onion address parse error: {0}")]
    OnionAddr(#[from] torut::onion::OnionAddressParseError),
}

pub type Result<T> = result::Result<T, PipeError>;

pub struct OnionPipeBuilder {
    temp_dir: path::PathBuf,
    exports: Vec<Export>,
    imports: Vec<Import>,
    secret_store: Option<secrets::SecretStore>,
}

impl OnionPipeBuilder {
    pub fn temp_dir(mut self, temp_dir: &str) -> OnionPipeBuilder {
        self.temp_dir = path::PathBuf::from(temp_dir);
        self
    }

    pub fn secrets_dir(mut self, secrets_dir: &str) -> OnionPipeBuilder {
        let secrets_dir = path::PathBuf::from(secrets_dir);
        self.secret_store = Some(secrets::SecretStore::new(secrets_dir.to_str().unwrap()));
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

    pub fn config(mut self, cfg: config::Config) -> Result<OnionPipeBuilder> {
        if let Some(secrets_dir) = cfg.secrets_dir {
            self = self.secrets_dir(&secrets_dir);
        }
        for cfg_export in cfg.exports {
            let export = match (cfg_export, self.secret_store.as_mut()).try_into() {
                Ok(item) => item,
                Err(err) => return Err(err),
            };
            self.exports.push(export);
        }
        for cfg_import in cfg.imports {
            let import = match cfg_import.try_into() {
                Ok(item) => item,
                Err(err) => return Err(err),
            };
            self.imports.push(import);
        }
        if let Some(temp_dir) = cfg.temp_dir {
            self = self.temp_dir(&temp_dir)
        }
        Ok(self)
    }

    pub async fn new(self) -> Result<OnionPipe> {
        let temp_dir = tempfile::tempdir_in(self.temp_dir)?;
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
    pub local_addr: net::SocketAddr,
    pub remote_key: onion::TorSecretKeyV3,
    pub remote_ports: Vec<u16>,
}

impl TryInto<Export> for (config::Export, Option<&mut secrets::SecretStore>) {
    type Error = PipeError;

    fn try_into(self) -> Result<Export> {
        let remote_key = match (self.0.service_name, self.1) {
            (Some(ref service_name), Some(secret_store)) => {
                let key_bytes = secret_store.ensure_service(service_name)?;
                torut::onion::TorSecretKeyV3::from(key_bytes)
            }
            (Some(_), None) => {
                return Err(PipeError::Config("secret store not configured".to_string()))
            }
            (None, _) => torut::onion::TorSecretKeyV3::generate(),
        };
        Ok(Export {
            local_addr: std::net::SocketAddr::from_str(self.0.local_addr.as_str())?,
            remote_key: remote_key,
            remote_ports: self.0.remote_ports,
        })
    }
}

pub struct Import {
    pub remote_addr: onion::OnionAddress,
    pub remote_port: u16,
    pub local_addr: net::SocketAddr,
}

impl TryInto<Import> for config::Import {
    type Error = PipeError;

    fn try_into(self) -> Result<Import> {
        let (remote_addr, remote_port) = parse_onion_address(&self.remote_addr)?;
        Ok(Import {
            remote_addr: torut::onion::OnionAddress::V3(remote_addr),
            remote_port: remote_port,
            local_addr: std::net::SocketAddr::from_str(self.local_addr.as_str())?,
        })
    }
}

fn parse_err(addr: &str) -> PipeError {
    return PipeError::Config(format!("invalid onion address {}", addr).to_string());
}

fn parse_onion_address(addr: &str) -> Result<(torut::onion::OnionAddressV3, u16)> {
    let re = Regex::new(r"^(?P<onion>[^\.]+)(\.onion)?(:(?P<port>\d+))$")
        .map_err(|_| parse_err(addr))?;
    match re.captures(addr) {
        Some(captures) => {
            let (remote_addr, remote_port) = match (captures.name("onion"), captures.name("port")) {
                (Some(onion), Some(port)) => (
                    torut::onion::OnionAddressV3::from_str(onion.as_str())?,
                    port.as_str().parse::<u16>().map_err(|_| parse_err(addr))?,
                ),
                (Some(onion), None) => (
                    torut::onion::OnionAddressV3::from_str(onion.as_str())?,
                    80u16,
                ),
                _ => return Err(parse_err(addr)),
            };
            Ok((remote_addr, remote_port))
        }
        None => Err(parse_err(addr)),
    }
}

pub enum Forward {
    Export(Export),
    Import(Import),
}

async fn on_event_noop(
    _: torut::control::AsyncEvent<'static>,
) -> result::Result<(), torut::control::ConnError> {
    Ok(())
}

impl OnionPipe {
    pub fn defaults() -> OnionPipeBuilder {
        OnionPipeBuilder {
            temp_dir: env::temp_dir(),
            exports: vec![],
            imports: vec![],
            secret_store: None,
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
            let remote_key = &export.remote_key;
            println!(
                "forward {} => {}:{}",
                export.local_addr,
                remote_key.public().get_onion_address(),
                export
                    .remote_ports
                    .iter()
                    .map(|port| port.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            );
            ac.add_onion_v3(
                remote_key,
                false,
                false,
                false,
                None,
                &mut export
                    .remote_ports
                    .iter()
                    .map(|port| (port.to_owned(), export.local_addr))
                    .collect::<Vec<_>>()
                    .iter(),
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
        eprintln!("interrupt received, shutting down");

        for i in 0..active_onions.len() {
            match ac.del_onion(active_onions[i].as_str()).await {
                Err(control::ConnError::IOError(io_err)) => {
                    if io_err.kind() == std::io::ErrorKind::ConnectionReset {
                        // Control connection may be lost here
                        break;
                    }
                    eprintln!("failed to delete onion: {:?}", io_err);
                }
                Err(err) => {
                    eprintln!("failed to delete onion: {:?}", err);
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

            println!("forward {} => {}", import_addr, import.local_addr,);
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
        let proxy_stream = match tokio::net::UnixStream::connect(socks_addr.as_str()).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("socks proxy connection failed: {}", e);
                continue;
            }
        };
        let remote_stream = match tokio_socks::tcp::Socks5Stream::connect_with_socket(
            proxy_stream,
            import_addr.as_str(),
        )
        .await
        {
            Ok(s) => s,
            Err(e) => {
                eprintln!("remote onion connection failed: {}", e);
                continue;
            }
        };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_into_export() {
        let export_config = config::Export {
            local_addr: "127.0.0.1:4566".to_string(),
            service_name: Some("some_service".to_string()),
            remote_ports: vec![4567],
        };
        let tmp_dir = tempfile::tempdir().unwrap();
        let secrets_dir = tmp_dir.path().join("secrets");
        let mut store = secrets::SecretStore::new(secrets_dir.to_str().unwrap());

        let export: Export = (export_config, Some(&mut store)).try_into().unwrap();
        assert_eq!("127.0.0.1:4566".parse(), Ok(export.local_addr));
        assert_eq!(
            export
                .remote_key
                .public()
                .get_onion_address()
                .get_address_without_dot_onion()
                .as_str()
                .len(),
            "wdz54gdzddxqigr27g5ivc4q3ekfrpmhe45yyb75kzhrkl577yalq7qd".len()
        );
        assert_eq!(export.remote_ports, vec![4567]);
        assert_eq!(store.list_services().unwrap(), vec!["some_service"]);

        // Test that secret store is consistent
        let export2_config = config::Export {
            local_addr: "127.0.0.1:4566".to_string(),
            service_name: Some("some_service".to_string()),
            remote_ports: vec![4567],
        };
        let export2: Export = (export2_config, Some(&mut store)).try_into().unwrap();
        assert_eq!(export.remote_key, export2.remote_key);
        assert_eq!(store.list_services().unwrap(), vec!["some_service"]);
    }

    #[test]
    fn try_into_export_new_onion() {
        let export_config = config::Export {
            local_addr: "127.0.0.1:4566".to_string(),
            service_name: None,
            remote_ports: vec![4567],
        };
        let tmp_dir = tempfile::tempdir().unwrap();
        let secrets_dir = tmp_dir.path().join("secrets");
        let mut store = secrets::SecretStore::new(secrets_dir.to_str().unwrap());

        let export: Export = (export_config, Some(&mut store)).try_into().unwrap();
        assert_eq!("127.0.0.1:4566".parse(), Ok(export.local_addr));
        assert_eq!(
            export
                .remote_key
                .public()
                .get_onion_address()
                .get_address_without_dot_onion()
                .as_str()
                .len(),
            56
        );
        assert_eq!(export.remote_ports, vec![4567]);
    }

    #[test]
    fn try_into_export_unix() {
        let export_config = config::Export {
            local_addr: "unix:/tmp/foo.sock".to_string(),
            service_name: None,
            remote_ports: vec![4567],
        };
        let tmp_dir = tempfile::tempdir().unwrap();
        let secrets_dir = tmp_dir.path().join("secrets");
        let mut store = secrets::SecretStore::new(secrets_dir.to_str().unwrap());

        let result: Result<Export> = (export_config, Some(&mut store)).try_into();
        // TODO: Improve torut to support local unix sockets.
        assert!(matches!(result, Err(PipeError::ParseAddr(_))));
    }
}
