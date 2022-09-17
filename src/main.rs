use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::net::UnixStream;

use libtor::{HiddenServiceVersion, Tor, TorAddress, TorFlag};

use torut::control::UnauthenticatedConn;

struct TorHandle {
    join_handle: Option<std::thread::JoinHandle<std::result::Result<u8, libtor::Error>>>,
}

impl TorHandle {
    fn new(jh: std::thread::JoinHandle<std::result::Result<u8, libtor::Error>>) -> TorHandle {
        TorHandle {
            join_handle: Some(jh),
        }
    }
}

impl Future for TorHandle {
    type Output = std::result::Result<u8, libtor::Error>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.join_handle.as_ref().unwrap().is_finished() {
            return Poll::Pending;
        }
        println!("finished, joining");
        Poll::Ready(
            self.join_handle
                .take()
                .unwrap()
                .join()
                .expect("failed to join Tor thread"),
        )
    }
}

struct Error {
    kind: ErrorKind,
}

enum ErrorKind {
    ControlPortTimeout,
}

type Result<T> = std::result::Result<T, Error>;

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
    Err(Error {
        kind: ErrorKind::ControlPortTimeout,
    })
}

async fn app() -> Result<()> {
    wait_for_file("/tmp/tor-rust/control.sock").await?;
    let s = UnixStream::connect("/tmp/tor-rust/control.sock")
        .await
        .unwrap();
    let mut utc = UnauthenticatedConn::new(s);
    let proto_info = utc.load_protocol_info().await.unwrap();
    println!("{:?}", proto_info);
    Ok(())
}

#[tokio::main]
async fn main() {
    let tor_handle = TorHandle::new(
        Tor::new()
            .flag(TorFlag::ControlSocket("/tmp/tor-rust/control.sock".into()))
            .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
            .flag(TorFlag::HiddenServiceDir("/tmp/tor-rust/hs-dir".into()))
            .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
            .flag(TorFlag::HiddenServicePort(
                TorAddress::Port(8000),
                None.into(),
            ))
            .start_background(),
    );

    tokio::spawn(app());
    tor_handle.await;
    std::process::exit(1);
}
