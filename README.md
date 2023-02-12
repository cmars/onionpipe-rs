# onionpipe

Onion addresses for anything.

`onionpipe` forwards ports on the local host to remote Onion addresses as Tor
hidden services and vice-versa.

### Why would I want to use this?

onionpipe is a decentralized way to create virtually unstoppable global network
tunnels.

For example, you might want to securely publish and access a personal service
from anywhere in the world, across all sorts of network obstructions -- your
ISP doesn't allow ingress traffic to your home lab, your clients might be in
heavily firewalled environments (public WiFi, mobile tether), etc.

With onionpipe, that service doesn't need a public IPv4 or IPv6 ingress. You
can publish services with a globally-unique persistent onion address, and share
access securely and privately to your own allowlist of authorized keys.

### What can I do with it right now?

Basic operation with a config file. Try, for example:

    cargo run -- --config examples/config.json

This example forwards a local port 4566 to a remote port 4567 on an ephemeral
onion address, while simultaneously forwarding a remote Tor project HTTP server
to local port 8080.

### What's the plan?

Config file format and API are currently unstable. No compatibility guarantees
at this time.

- CLI compatibility with the [Go implementation](https://github.com/cmars/onionpipe).
- Rust library distribution with simplified high-level port-forwarding API
- A GUI distribution possibly based on Tauri
- Cross-platform distribution of the above: Linux, macOS, Windows on popular architectures

