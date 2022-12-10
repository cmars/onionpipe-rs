use std::net;
use std::str::FromStr;

use onionpipe::{Export, Import, OnionPipe};

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
