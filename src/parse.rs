// Some parsing functions adapted from https://github.com/zupzup/rust-nom-parsing, which is
// licensed:
//
// Copyright 2020 Mario Zupan <mario@zupzup.org>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod parse {

    use std::str::FromStr;

    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{alphanumeric1, one_of},
        combinator::{eof, opt},
        error::{context, VerboseError},
        multi::{count, many_m_n, separated_list1},
        sequence::{preceded, terminated, tuple},
        Err as NomErr, Finish, IResult,
    };

    #[derive(Debug, PartialEq, Eq)]
    pub struct ParseError(String);

    #[derive(Debug, PartialEq, Eq)]
    pub struct ExportLocalTCPAddr {
        host: Option<Host>,
        port: u16,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum Host {
        HOST(String),
        IP4([u8; 4]),
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum ExportLocalAddr {
        TCP(ExportLocalTCPAddr),
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

    #[derive(Debug, PartialEq, Eq)]
    pub struct ImportRemoteAddr {
        onion: String,
        port: Option<u16>,
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

    #[derive(Debug, PartialEq, Eq)]
    pub struct ImportForward {
        remote: ImportRemoteAddr,
        local: Option<ImportLocalAddr>,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum Forward {
        Export(ExportForward),
        Import(ImportForward),
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

    type Res<T, U> = IResult<T, U, VerboseError<T>>;

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

    fn ip(input: &str) -> Res<&str, Host> {
        context(
            "ip",
            tuple((count(terminated(ip_num, tag(".")), 3), ip_num)),
        )(input)
        .map(|(next_input, res)| {
            let mut result: [u8; 4] = [0, 0, 0, 0];
            res.0
                .into_iter()
                .enumerate()
                .for_each(|(i, v)| result[i] = v);
            result[3] = res.1;
            (next_input, Host::IP4(result))
        })
    }

    fn ip_num(input: &str) -> Res<&str, u8> {
        context("ip number", n_to_m_digits(1, 3))(input).and_then(|(next_input, result)| {
            match result.parse::<u8>() {
                Ok(n) => Ok((next_input, n)),
                Err(_) => Err(NomErr::Error(VerboseError { errors: vec![] })),
            }
        })
    }

    fn port(input: &str) -> Res<&str, u16> {
        context("port", n_to_m_digits(2, 5))(input).and_then(|(next_input, res)| {
            match res.parse::<u16>() {
                Ok(n) => Ok((next_input, n)),
                Err(_) => Err(NomErr::Error(VerboseError { errors: vec![] })),
            }
        })
    }

    fn ports(input: &str) -> Res<&str, Vec<u16>> {
        context("ports", separated_list1(tag(","), port))(input).map(|(next_input, res)| {
            let mut result: Vec<u16> = vec![0; res.len()];
            res.into_iter().enumerate().for_each(|(i, v)| result[i] = v);
            (next_input, result)
        })
    }

    fn n_to_m_digits<'a>(n: usize, m: usize) -> impl FnMut(&'a str) -> Res<&str, String> {
        move |input| {
            many_m_n(n, m, one_of("0123456789"))(input)
                .map(|(next_input, result)| (next_input, result.into_iter().collect()))
        }
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
                    "Parse error:\nEof at: ,81,82\nin section 'forward', at: 80,81,82\n"
                        .to_string()
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
}
