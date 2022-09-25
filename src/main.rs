use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::os::unix::fs::PermissionsExt;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PipeError {
    #[error("timeout connecting to tor control socket")]
    ConnTimeout,
    #[error("failed to connect to tor control socket")]
    Conn(#[from] torut::control::ConnError),
    #[error("i/o error")]
    IO(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, PipeError>;

pub struct OnionPipeBuilder {
    temp_path: std::path::PathBuf,
    exports: Vec<Export>,
    imports: Vec<Import>,
}

impl OnionPipeBuilder {
    pub fn temp_path(mut self, temp_path: &str) -> OnionPipeBuilder {
        self.temp_path = std::path::PathBuf::from(temp_path);
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
        tokio::fs::set_permissions(data_dir.as_path(), std::fs::Permissions::from_mode(0o700))
            .await?;
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
    local_addr: SocketAddr,
    remote_addr: Option<torut::onion::OnionAddress>,
    remote_port: u16,
}

pub struct Import {
    remote_addr: torut::onion::OnionAddress,
    remote_port: u16,
    local_addr: SocketAddr,
}

impl OnionPipe {
    pub fn defaults() -> OnionPipeBuilder {
        OnionPipeBuilder {
            temp_path: std::env::temp_dir(),
            exports: vec![],
            imports: vec![],
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.start_tor();

        wait_for_file(&self.control_sock).await?;
        let s = tokio::net::UnixStream::connect(&self.control_sock).await?;
        let mut utc = torut::control::UnauthenticatedConn::new(s);
        utc.authenticate(&torut::control::TorAuthData::Null).await?;
        let mut ac = utc.into_authenticated().await;
        ac.set_async_event_handler(Some(|_| async move { Ok(()) }));
        ac.take_ownership().await?;

        let key = torut::onion::TorSecretKeyV3::generate();
        println!("using onion address: {}", key.public().get_onion_address());

        ac.add_onion_v3(
            &key,
            false,
            false,
            false,
            None,
            &mut [(
                8000,
                SocketAddr::new(IpAddr::from(Ipv4Addr::new(127, 0, 0, 1)), 8000),
            )]
            .iter(),
        )
        .await?;

        tokio::signal::ctrl_c().await?;
        println!("interrupt received, shutting down");

        ac.del_onion(
            &key.public()
                .get_onion_address()
                .get_address_without_dot_onion(),
        )
        .await?;

        drop(ac);
        self.temp_dir.take().unwrap().close()?;
        Ok(())
    }

    fn start_tor(&self) -> () {
        libtor::Tor::new()
            .flag(libtor::TorFlag::ControlSocket(
                self.control_sock.as_str().into(),
            ))
            .flag(libtor::TorFlag::DataDirectory(
                self.data_dir.as_str().into(),
            ))
            .start_background();
    }
}

#[tokio::main]
async fn main() {
    let mut onion_pipe = OnionPipe::defaults().new().await.unwrap();
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
