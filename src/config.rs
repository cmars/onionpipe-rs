use serde::{Deserialize, Serialize};

use crate as onionpipe;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Config {
    pub temp_dir: Option<String>,
    pub secrets_dir: Option<String>,
    pub exports: Vec<Export>,
    pub imports: Vec<Import>,
}

impl TryInto<Config> for Vec<String> {
    type Error = onionpipe::PipeError;

    fn try_into(self) -> onionpipe::Result<Config> {
        let mut cfg: Config = Config {
            temp_dir: None,
            secrets_dir: None,
            exports: vec![],
            imports: vec![],
        };
        for forward_expr in self {
            let parsed_forward = forward_expr.parse::<onionpipe::parse::Forward>()?;
            let cfg_forward: Forward = parsed_forward.into();
            match cfg_forward {
                Forward::Import(import) => cfg.imports.push(import),
                Forward::Export(export) => cfg.exports.push(export),
            }
        }
        Ok(cfg)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Export {
    pub local_addr: String,
    pub service_name: Option<String>,
    pub remote_ports: Vec<u16>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Import {
    pub remote_addr: String,
    pub local_addr: String,
}

pub enum Forward {
    Import(Import),
    Export(Export),
}

#[cfg(test)]
mod tests {
    use serde_json;

    use super::*;

    #[test]
    fn try_config_parse() {
        let json_str = r#"
            {
              "temp_dir": "/tmp/foo",
              "secrets_dir": "/tmp/secrets",
              "exports": [{
                "local_addr": "127.0.0.1:4566",
                "service_name": "some_service",
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
                secrets_dir: Some("/tmp/secrets".to_string()),
                exports: vec![Export {
                    local_addr: "127.0.0.1:4566".to_string(),
                    service_name: Some("some_service".to_string()),
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
}
