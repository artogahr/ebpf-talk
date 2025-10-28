# eBPF Talk - Rust Meetup

Materials for my talk on eBPF, Aya and network programming in Rust.

Based on my master's thesis work with [lockne](https://github.com/your-username/lockne) - dynamic per-application VPN tunneling using eBPF.

## Structure

```
ebpf-talk/
├── slides/              # presenterm slides
└── demos/               # working demos
    ├── 01-packet-monitor/   # TC classifier (basic)
    └── 02-process-tracker/  # Tracepoint (more interesting)
```

## Quick Start

### Prerequisites

Using Nix flakes:
```bash
nix develop
```

This gives you:
- Rust toolchains (stable + nightly with rust-src)
- bpf-linker
- cargo-generate
- presenterm

### Build Demos

```bash
cd demos/01-packet-monitor
cargo build

cd ../02-process-tracker
cargo build
```

### Run Demos

**Packet Monitor (TC Classifier):**
```bash
cd demos/01-packet-monitor
sudo ./target/debug/demo --iface lo --duration 10
# generates traffic: ping localhost
```

**Process Tracker (Tracepoint):** - more interesting!
```bash
cd demos/02-process-tracker
RUST_LOG=info sudo -E ./target/debug/file-tracker --duration 30
# in another terminal: ls, curl http://example.com, python3 --version
```

The process tracker will show every command execution in real-time.

### View Slides

```bash
cd slides
presenterm presentation.md
```

Arrow keys to navigate, `q` to quit.

## What's Different About These Demos

Both demos show different eBPF program types:

- **01-packet-monitor**: TC (Traffic Control) classifier on network egress
- **02-process-tracker**: Tracepoint attached to `sched_process_exec`

This demonstrates that eBPF isn't just for networking - it's a general kernel programming framework. My thesis project actually combines both approaches (TC for packets + cgroup for process tracking).

## About the Talk

20 minute presentation covering:
- What is eBPF and why it's cool
- Why Rust + Aya is better than C + BCC
- Real challenges from my thesis work
- Live demo (process tracker)
- Performance results

The slides include actual code from my thesis and comparisons between C/BCC and Rust/Aya implementations.
