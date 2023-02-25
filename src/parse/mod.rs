use std::error;
use std::fmt;
use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::alphanumeric1,
    combinator::{eof, opt},
    error::context,
    multi::separated_list1,
    sequence::{preceded, terminated, tuple},
    Finish,
};

use crate::config;

mod addr;

use addr::{ip, port};
pub use addr::{Host, Res};

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for ParseError {}

#[derive(Debug, PartialEq, Eq)]
pub struct ExportLocalTCPAddr {
    host: Option<Host>,
    port: u16,
}

impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Host::HOST(host) => write!(f, "{}", host),
            Host::IP4(ip) => write!(f, "{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExportLocalAddr {
    TCP(ExportLocalTCPAddr),
}

impl fmt::Display for ExportLocalAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExportLocalAddr::TCP(addr) => write!(
                f,
                "{}:{}",
                addr.host.as_ref().unwrap_or(&Host::IP4([127, 0, 0, 1])),
                addr.port
            ),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExportRemoteAddr {
    onion_alias: Option<String>,
    ports: Vec<u16>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExportForward {
    local: ExportLocalAddr,
    remote: Option<ExportRemoteAddr>,
}

impl From<ExportForward> for config::Export {
    fn from(export: ExportForward) -> Self {
        config::Export {
            local_addr: format!("{}", export.local),
            remote_onion_secret_key: None,
            // TODO: support for onion key mgmt
            remote_ports: match export.remote {
                Some(remote) => remote.ports,
                None => vec![80u16],
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ImportRemoteAddr {
    onion: String,
    port: Option<u16>,
}

impl fmt::Display for ImportRemoteAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}",
            self.onion,
            match self.port {
                Some(port) => port,
                None => 80u16,
            }
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ImportLocalAddr {
    TCP(ImportLocalTCPAddr),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ImportLocalTCPAddr {
    host: Option<Host>,
    port: Option<u16>,
}

static LOCALHOST: &str = "127.0.0.1";

impl fmt::Display for ImportLocalAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ImportLocalAddr::TCP(tcp_addr) => write!(
                f,
                "{}:{}",
                match tcp_addr.host.as_ref() {
                    Some(host) => format!("{}", host),
                    None => LOCALHOST.to_string(),
                },
                tcp_addr.port.unwrap_or(80u16),
            ),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ImportForward {
    remote: ImportRemoteAddr,
    local: Option<ImportLocalAddr>,
}

impl From<ImportForward> for config::Import {
    fn from(import: ImportForward) -> Self {
        config::Import {
            remote_addr: format!("{}", import.remote),
            local_addr: format!(
                "{}",
                format!(
                    "{}",
                    import
                        .local
                        .unwrap_or(ImportLocalAddr::TCP(ImportLocalTCPAddr {
                            host: Some(Host::IP4([127, 0, 0, 1])),
                            port: Some(8080u16),
                        }))
                )
            ),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Forward {
    Export(ExportForward),
    Import(ImportForward),
}

impl From<Forward> for config::Forward {
    fn from(forward: Forward) -> Self {
        match forward {
            Forward::Export(export) => config::Forward::Export(export.into()),
            Forward::Import(import) => config::Forward::Import(import.into()),
        }
    }
}

impl FromStr for Forward {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match forward(s).finish() {
            Ok((_, forward)) => Ok(forward),
            Err(e) => Err(ParseError(e.to_string())),
        }
    }
}

fn forward(input: &str) -> Res<&str, Forward> {
    context(
        "forward",
        tuple((alt((import_forward, export_forward)), eof)),
    )(input)
    .map(|(next_input, res)| (next_input, res.0))
}

fn export_forward(input: &str) -> Res<&str, Forward> {
    context(
        "export forward",
        tuple((local_tcp_addr, opt(preceded(tag("~"), export_remote_addr)))),
    )(input)
    .map(|(next_input, res)| {
        let result: Forward = Forward::Export(ExportForward {
            local: ExportLocalAddr::TCP(res.0),
            remote: res.1,
        });
        (next_input, result)
    })
}

fn export_remote_addr(input: &str) -> Res<&str, ExportRemoteAddr> {
    context(
        "export remote addr",
        tuple((opt(terminated(alphanumeric1, tag(":"))), ports)),
    )(input)
    .map(|(next_input, res)| {
        let result: ExportRemoteAddr = ExportRemoteAddr {
            onion_alias: res.0.map(|s| s.to_string()),
            ports: res.1,
        };
        (next_input, result)
    })
}

fn local_tcp_addr(input: &str) -> Res<&str, ExportLocalTCPAddr> {
    context(
        "local tcp addr",
        tuple((opt(terminated(ip, tag(":"))), port)),
    )(input)
    .map(|(next_input, res)| {
        let result: ExportLocalTCPAddr = ExportLocalTCPAddr {
            host: res.0,
            port: res.1,
        };
        (next_input, result)
    })
}

fn import_forward(input: &str) -> Res<&str, Forward> {
    context(
        "import forward",
        tuple((
            import_remote_addr,
            opt(preceded(tag("~"), import_local_addr)),
        )),
    )(input)
    .map(|(next_input, res)| {
        let result: Forward = Forward::Import(ImportForward {
            remote: res.0,
            local: res.1,
        });
        (next_input, result)
    })
}

fn import_remote_addr(input: &str) -> Res<&str, ImportRemoteAddr> {
    context(
        "import remote addr",
        tuple((onion, opt(preceded(tag(":"), port)))),
    )(input)
    .map(|(next_input, res)| {
        let result: ImportRemoteAddr = ImportRemoteAddr {
            onion: res.0.to_owned(),
            port: res.1,
        };
        (next_input, result)
    })
}

fn onion(input: &str) -> Res<&str, String> {
    context("onion", terminated(alphanumeric1, tag(".onion")))(input)
        .map(|(next_input, res)| (next_input, res.to_owned()))
}

fn import_local_addr(input: &str) -> Res<&str, ImportLocalAddr> {
    context(
        "import local addr",
        alt((
            import_local_addr_host_port,
            import_local_addr_host_only,
            import_local_addr_port_only,
        )),
    )(input)
    .map(|(next_input, res)| {
        let result: ImportLocalAddr = ImportLocalAddr::TCP(res);
        (next_input, result)
    })
}

fn import_local_addr_host_port(input: &str) -> Res<&str, ImportLocalTCPAddr> {
    context("import local addr host:port", tuple((ip, tag(":"), port)))(input).map(
        |(next_input, res)| {
            (
                next_input,
                ImportLocalTCPAddr {
                    host: Some(res.0),
                    port: Some(res.2),
                },
            )
        },
    )
}

fn import_local_addr_host_only(input: &str) -> Res<&str, ImportLocalTCPAddr> {
    context("import local addr host only", ip)(input).map(|(next_input, res)| {
        (
            next_input,
            ImportLocalTCPAddr {
                host: Some(res),
                port: None,
            },
        )
    })
}

fn import_local_addr_port_only(input: &str) -> Res<&str, ImportLocalTCPAddr> {
    context("import local addr host only", port)(input).map(|(next_input, res)| {
        (
            next_input,
            ImportLocalTCPAddr {
                host: None,
                port: Some(res),
            },
        )
    })
}

fn ports(input: &str) -> Res<&str, Vec<u16>> {
    context("ports", separated_list1(tag(","), port))(input).map(|(next_input, res)| {
        let mut result: Vec<u16> = vec![0; res.len()];
        res.into_iter().enumerate().for_each(|(i, v)| result[i] = v);
        (next_input, result)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{
        error::{ErrorKind, VerboseError, VerboseErrorKind},
        Err as NomErr,
    };

    #[test]
    fn test_local_tcp_addr() {
        assert_eq!(
            local_tcp_addr("127.0.0.1:4567~"),
            Ok((
                "~",
                ExportLocalTCPAddr {
                    host: Some(Host::IP4([127, 0, 0, 1])),
                    port: 4567
                }
            ))
        );
        assert_eq!(
            local_tcp_addr("6667~"),
            Ok((
                "~",
                ExportLocalTCPAddr {
                    host: None,
                    port: 6667
                }
            ))
        );
    }

    #[test]
    fn test_export_forward_valid() {
        assert_eq!(
            forward("80"),
            Ok((
                "",
                Forward::Export(ExportForward {
                    local: ExportLocalAddr::TCP(ExportLocalTCPAddr {
                        host: None,
                        port: 80,
                    }),
                    remote: None,
                })
            ))
        );
        assert_eq!(
            forward("1.2.3.4:80"),
            Ok((
                "",
                Forward::Export(ExportForward {
                    local: ExportLocalAddr::TCP(ExportLocalTCPAddr {
                        host: Some(Host::IP4([1, 2, 3, 4])),
                        port: 80,
                    }),
                    remote: None,
                })
            ))
        );
        assert_eq!(
            forward("80~80"),
            Ok((
                "",
                Forward::Export(ExportForward {
                    local: ExportLocalAddr::TCP(ExportLocalTCPAddr {
                        host: None,
                        port: 80,
                    }),
                    remote: Some(ExportRemoteAddr {
                        onion_alias: None,
                        ports: vec![80],
                    }),
                })
            ))
        );
        assert_eq!(
            forward("80~80,81,8080,28000"),
            Ok((
                "",
                Forward::Export(ExportForward {
                    local: ExportLocalAddr::TCP(ExportLocalTCPAddr {
                        host: None,
                        port: 80,
                    }),
                    remote: Some(ExportRemoteAddr {
                        onion_alias: None,
                        ports: vec![80, 81, 8080, 28000],
                    }),
                })
            ))
        );
        assert_eq!(
            forward("80~mastodon:80,81,8080,28000"),
            Ok((
                "",
                Forward::Export(ExportForward {
                    local: ExportLocalAddr::TCP(ExportLocalTCPAddr {
                        host: None,
                        port: 80,
                    }),
                    remote: Some(ExportRemoteAddr {
                        onion_alias: Some("mastodon".to_string()),
                        ports: vec![80, 81, 8080, 28000],
                    }),
                })
            ))
        );
        assert_eq!(
            forward("0.0.0.0:80~mastodon:80,81,8080,28000"),
            Ok((
                "",
                Forward::Export(ExportForward {
                    local: ExportLocalAddr::TCP(ExportLocalTCPAddr {
                        host: Some(Host::IP4([0, 0, 0, 0])),
                        port: 80,
                    }),
                    remote: Some(ExportRemoteAddr {
                        onion_alias: Some("mastodon".to_string()),
                        ports: vec![80, 81, 8080, 28000],
                    }),
                })
            ))
        );
    }

    #[test]
    fn test_export_forward_invalid() {
        assert_eq!(
            forward("1.2.3.4"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    (".2.3.4", VerboseErrorKind::Nom(ErrorKind::OneOf)),
                    (".2.3.4", VerboseErrorKind::Nom(ErrorKind::ManyMN)),
                    ("1.2.3.4", VerboseErrorKind::Context("port")),
                    ("1.2.3.4", VerboseErrorKind::Context("local tcp addr")),
                    ("1.2.3.4", VerboseErrorKind::Context("export forward")),
                    ("1.2.3.4", VerboseErrorKind::Nom(ErrorKind::Alt)),
                    ("1.2.3.4", VerboseErrorKind::Context("forward"))
                ]
            }))
        );
        // TODO: improve nom error messages, they're kinda terrible
        assert_eq!(
            "80,81,82".parse::<Forward>(),
            Err(ParseError(
                "Parse error:\nEof at: ,81,82\nin section 'forward', at: 80,81,82\n".to_string()
            ))
        );
        assert_eq!(
                "80,81,82~8000,8001,8002".parse::<Forward>(),
                Err(ParseError("Parse error:\nEof at: ,81,82~8000,8001,8002\nin section 'forward', at: 80,81,82~8000,8001,8002\n".to_string()))
            );
        assert_eq!(
                "".parse::<Forward>(),
                Err(ParseError(
                        "Parse error:\nOneOf at: \nManyMN at: \nin section 'port', at: \nin section 'local tcp addr', at: \nin section 'export forward', at: \nAlt at: \nin section 'forward', at: \n".to_string()
                ))
            );
        assert_eq!(
                "10.0.0.1:8080~192.168.1.1:8080".parse::<Forward>(),
                Err(ParseError("Parse error:\nEof at: .168.1.1:8080\nin section 'forward', at: 10.0.0.1:8080~192.168.1.1:8080\n".to_string()))
            );
    }

    #[test]
    fn test_import_forward_valid() {
        assert_eq!(
            "xyz123.onion".parse::<Forward>(),
            Ok(Forward::Import(ImportForward {
                remote: ImportRemoteAddr {
                    onion: "xyz123".to_string(),
                    port: None,
                },
                local: None,
            }))
        );
        assert_eq!(
            "xyz123.onion:9001".parse::<Forward>(),
            Ok(Forward::Import(ImportForward {
                remote: ImportRemoteAddr {
                    onion: "xyz123".to_string(),
                    port: Some(9001),
                },
                local: None,
            }))
        );
        assert_eq!(
            "xyz123.onion:9001~9002".parse::<Forward>(),
            Ok(Forward::Import(ImportForward {
                remote: ImportRemoteAddr {
                    onion: "xyz123".to_string(),
                    port: Some(9001),
                },
                local: Some(ImportLocalAddr::TCP(ImportLocalTCPAddr {
                    host: None,
                    port: Some(9002),
                })),
            }))
        );
        assert_eq!(
            "xyz123.onion:9001~172.18.0.1".parse::<Forward>(),
            Ok(Forward::Import(ImportForward {
                remote: ImportRemoteAddr {
                    onion: "xyz123".to_string(),
                    port: Some(9001),
                },
                local: Some(ImportLocalAddr::TCP(ImportLocalTCPAddr {
                    host: Some(Host::IP4([172, 18, 0, 1])),
                    port: None,
                })),
            }))
        );
    }

    #[test]
    fn test_import_forward_invalid() {
        assert_eq!(
                "xyz123".parse::<Forward>(),
                Err(ParseError("Parse error:\nOneOf at: xyz123\nManyMN at: xyz123\nin section 'port', at: xyz123\nin section 'local tcp addr', at: xyz123\nin section 'export forward', at: xyz123\nAlt at: xyz123\nin section 'forward', at: xyz123\n".to_string()))
            );
        assert_eq!(
                "xyz123.shallot".parse::<Forward>(),
                Err(ParseError("Parse error:\nOneOf at: xyz123.shallot\nManyMN at: xyz123.shallot\nin section 'port', at: xyz123.shallot\nin section 'local tcp addr', at: xyz123.shallot\nin section 'export forward', at: xyz123.shallot\nAlt at: xyz123.shallot\nin section 'forward', at: xyz123.shallot\n".to_string()))
            );
        assert_eq!(
                "xyz123.onion~abc123.onion".parse::<Forward>(),
                Err(ParseError("Parse error:\nEof at: ~abc123.onion\nin section 'forward', at: xyz123.onion~abc123.onion\n".to_string()))
            );
    }
}
