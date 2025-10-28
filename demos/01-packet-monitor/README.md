# Packet Monitor

TC (Traffic Control) classifier that logs packets passing through a network interface.

Generated from `aya-template` with minimal modifications.

## What It Does

Attaches an eBPF program to the egress (outgoing) side of a network interface and logs every packet that passes through.

## Building

```bash
cargo build
```

Requires:
- `bpf-linker` (for building eBPF code)
- `rustup` with nightly + rust-src (for eBPF target)

If using the nix flake in the repo root, you get all this automatically.

## Running

```bash
sudo ./target/debug/demo --iface lo --duration 10
```

Options:
- `--iface <name>` - Network interface to attach to (default: eno1)
- `--duration <secs>` - Auto-exit after N seconds (default: run until Ctrl-C)

### Testing

Use loopback interface for safety:
```bash
sudo ./target/debug/demo --iface lo --duration 10
```

Then in another terminal, generate traffic:
```bash
ping localhost
curl http://localhost
```

You'll see log messages for each packet.

## How It Works

**Kernel side** (`demo-ebpf/src/main.rs`):
- `#[classifier]` macro marks this as a TC classifier
- Receives `TcContext` with packet data
- Logs "received a packet" for each packet
- Returns `TC_ACT_PIPE` (pass packet through)

**User side** (`demo/src/main.rs`):
- Loads compiled eBPF bytecode
- Attaches to specified interface using TC egress hook
- Sets up async logger to receive kernel logs
- Waits for Ctrl-C or timeout

## Notes

- Needs `sudo` because eBPF requires `CAP_BPF` capability
- The `--duration` flag was added to make testing easier
- By default attaches to `eno1` - use `--iface lo` for loopback
- Egress = outgoing traffic (could also use ingress for incoming)

This is the most basic eBPF networking demo. Real tools would actually parse the packet data, not just log that packets exist.
