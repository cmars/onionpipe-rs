use std;
use std::str::FromStr;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate as onionpipe;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Config {
    pub temp_dir: Option<String>,
    pub exports: Vec<Export>,
    pub imports: Vec<Import>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Export {
    pub local_addr: String,
    pub remote_onion_alias: Option<String>,
    pub remote_ports: Vec<u16>,
}

impl TryInto<onionpipe::Export> for Export {
    type Error = onionpipe::PipeError;

    fn try_into(self) -> onionpipe::Result<onionpipe::Export> {
        Ok(onionpipe::Export {
            local_addr: std::net::SocketAddr::from_str(self.local_addr.as_str())?,
            remote_onion_alias: self.remote_onion_alias,
            remote_ports: self.remote_ports,
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Import {
    pub remote_addr: String,
    pub local_addr: String,
}

impl TryInto<onionpipe::Import> for Import {
    type Error = onionpipe::PipeError;

    fn try_into(self) -> onionpipe::Result<onionpipe::Import> {
        let (remote_addr, remote_port) = parse_onion_address(&self.remote_addr)?;
        Ok(onionpipe::Import {
            remote_addr: torut::onion::OnionAddress::V3(remote_addr),
            remote_port: remote_port,
            local_addr: std::net::SocketAddr::from_str(self.local_addr.as_str())?,
        })
    }
}

pub enum Forward {
    Import(Import),
    Export(Export),
}

fn parse_err(addr: &str) -> onionpipe::PipeError {
    return onionpipe::PipeError::Config(format!("invalid onion address {}", addr).to_string());
}

fn parse_onion_address(addr: &str) -> onionpipe::Result<(torut::onion::OnionAddressV3, u16)> {
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

#[cfg(test)]
mod tests {
    use serde_json;

    use super::*;
    use onionpipe;

    #[test]
    fn try_config_parse() {
        let json_str = r#"
            {
              "temp_dir": "/tmp/foo",
              "exports": [{
                "local_addr": "127.0.0.1:4566",
                "remote_onion_alias": "scallion",
                "remote_ports": [4567]
              }],
              "imports": [{
                "remote_addr": "2gzyxa5ihm7nsggfxnu52rck2vv4rvmdlkiu3zzui5du4xyclen53wid.onion:80",
                "local_addr": "127.0.0.1:8080"
              }]
            }"#;
        let config: Config = serde_json::from_str(json_str).unwrap();
        assert_eq!(
            config,
            Config {
                temp_dir: Some("/tmp/foo".to_string()),
                exports: vec![Export {
                    local_addr: "127.0.0.1:4566".to_string(),
                    remote_onion_alias: Some("scallion".to_string()),
                    remote_ports: vec![4567],
                }],
                imports: vec![Import {
                    remote_addr:
                        "2gzyxa5ihm7nsggfxnu52rck2vv4rvmdlkiu3zzui5du4xyclen53wid.onion:80"
                            .to_string(),
                    local_addr: "127.0.0.1:8080".to_string(),
                }],
            }
        );
    }

    #[test]
    fn try_into_export() {
        let export_config = Export {
            local_addr: "127.0.0.1:4566".to_string(),
            remote_onion_alias: Some("scallion".to_string()),
            remote_ports: vec![4567],
        };
        let export: onionpipe::Export = export_config.try_into().unwrap();
        assert_eq!("127.0.0.1:4566".parse(), Ok(export.local_addr));
        assert_eq!(export.remote_onion_alias, Some("scallion".to_string()));
        assert_eq!(export.remote_ports, vec![4567]);
    }

    #[test]
    fn try_into_export_new_onion() {
        let export_config = Export {
            local_addr: "127.0.0.1:4566".to_string(),
            remote_onion_alias: None,
            remote_ports: vec![4567],
        };
        let export: onionpipe::Export = export_config.try_into().unwrap();
        assert_eq!("127.0.0.1:4566".parse(), Ok(export.local_addr));
        assert_eq!(export.remote_onion_alias, None);
        assert_eq!(export.remote_ports, vec![4567]);
    }

    #[test]
    fn try_into_export_unix() {
        let export_config = Export {
            local_addr: "unix:/tmp/foo.sock".to_string(),
            remote_onion_alias: Some("scallion".to_string()),
            remote_ports: vec![4567],
        };
        let result: Result<onionpipe::Export, onionpipe::PipeError> = export_config.try_into();
        // TODO: Improve torut to support local unix sockets.
        assert!(matches!(result, Err(onionpipe::PipeError::ParseAddr(_))));
    }
}
