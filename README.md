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

Nothing yet. This is still in development.

### What's the plan?

- CLI compatibility with the [Go implementation](https://github.com/cmars/onionpipe).
- Rust library distribution with simplified high-level port-forwarding API
- A GUI distribution possibly based on Tauri
- Cross-platform distribution of the above: Linux, macOS, Windows on popular architectures

