# onionpipe

Onion addresses for anything.

`onionpipe` forwards ports on the local host to remote Onion addresses as Tor
hidden services and vice-versa.

## Why would I want to use this?

onionpipe is a decentralized way to create virtually unstoppable global network
tunnels.

For example, you might want to securely publish and access a personal service
from anywhere in the world, across all sorts of network obstructions -- your
ISP doesn't allow ingress traffic to your home lab, your clients might be in
heavily firewalled environments (public WiFi, mobile tether), etc.

With onionpipe, that service doesn't need a public IPv4 or IPv6 ingress. You
can publish services with a globally-unique persistent onion address, and share
access securely and privately to your own allowlist of authorized keys.

## How do I install it?

On Linux (and probably macOS? untested atm):

    cargo install onionpipe

## What can I do with it right now?

onionpipe sets up socket forwarding tunnels. It's like `socat(1)`, for onions.

### Export services on local networks to onion addresses

Export localhost port 8000 to a temporary, one-time remote onion address. Default is port 80 on the onion service.

```
onionpipe 8000
```

The forwarding address is printed to the log output:

```
Feb 26 09:41:18.267 [notice] Tor 0.4.7.10 running on Linux with Libevent 2.1.12-stable, OpenSSL 1.1.1t, Zlib 1.2.11, Liblzma N/A, Libzstd N/A and Glibc 2.35 as libc.
Feb 26 09:41:18.267 [notice] Tor can't help you if you use it wrong! Learn how to be safe at https://support.torproject.org/faq/staying-anonymous/
Feb 26 09:41:18.267 [notice] Configuration file "/home/c/.torrc" not present, using reasonable defaults.
Feb 26 09:41:18.271 [notice] Opening Socks listener on /run/user/1000/.tmpfviq7N/data/socks.sock
Feb 26 09:41:18.271 [notice] Opened Socks listener connection (ready) on /run/user/1000/.tmpfviq7N/data/socks.sock
Feb 26 09:41:18.271 [notice] Opening Control listener on /run/user/1000/.tmpfviq7N/data/control.sock
Feb 26 09:41:18.271 [notice] Opened Control listener connection (ready) on /run/user/1000/.tmpfviq7N/data/control.sock
forward 127.0.0.1:8000 => pqksfxbpraiwklpx7ihu7yu7vlpkpromqojyn6goo2fl6wemi4dkieqd.onion:80
```

Port forwarding can be mapped. This exports localhost port 8443 to temporary remote onion port 443. `~` is shorthand
for the forward between source~destination.

```
onionpipe 8443~443
```

Local addresses may be bound. This forwards a specific interface address to an onion:

```
onionpipe 10.0.0.7:8443~443
```

### Import onion services


This imports an Onion site to a local listener on port 8000.

```
onionpipe ddosxlvzzow7scc7egy75gpke54hgbg2frahxzaw6qq5osnzm7wistid.onion~8000
```

Import an Onion site to a specific address. Useful for setting up an intranet or clearnet ingress to the onion service.

```
onionpipe ddosxlvzzow7scc7egy75gpke54hgbg2frahxzaw6qq5osnzm7wistid.onion~0.0.0.0:8000
```

### Config file operation

All the above and more can be expressed with a JSON configuration file. See [Config](https://docs.rs/onionpipe/0.3.0/onionpipe/config/struct.Config.html) Rust docs and [an example config.json](examples/config.json) for details.

```
onionpipe --config config.json
```

## TODOs

- Security review. Rust code review, I'm kind of new to the language.
- CLI compatibility with the [Go implementation](https://github.com/cmars/onionpipe). What's still missing?
  - Onion service key management
  - Client authentication & key management
  - More Tor options like anonymous vs fast, bridge support. Vanguard integration.
  - UNIX socket support. Doable but a dependency will need some enhancement (torut)
- Cross-platform distribution of the above: Linux, macOS, Windows on popular architectures
  - Distributions on Docker, NixOS (flake), Homebrew, maybe Choco?

## More ideas!

- GUI front-end, possibly based on Tauri
- cwtch integration
- daemon mode & forwarding control API
- Kubernetes CRD: OnionService (which could use the control API)
- Arti-based fork, when Arti supports hidden services

