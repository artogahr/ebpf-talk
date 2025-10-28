# Demos

Two demos showing different eBPF capabilities and program types.

## Demo 1: Packet Monitor (TC Classifier)

**What it does:** Attaches to network interface and logs every packet.

**eBPF type:** TC (Traffic Control) classifier  
**Hook point:** Network stack egress  
**Generated from:** `aya-template`

```bash
cd 01-packet-monitor
cargo build
sudo ./target/debug/demo --iface lo --duration 10
```

Then generate some traffic (ping localhost, curl, etc).

This is the basic "hello world" of eBPF networking - shows how TC classifiers work but isn't super interesting on its own.

## Demo 2: Process Tracker (Tracepoint) ⭐

**What it does:** Watches the scheduler and logs every new process execution.

**eBPF type:** Tracepoint  
**Hook point:** `sched/sched_process_exec`  
**Why it's cool:** Shows eBPF for observability, not just networking

```bash
cd 02-process-tracker
cargo build
RUST_LOG=info sudo -E ./target/debug/file-tracker --duration 30
```

**Important:** You need `RUST_LOG=info` and `sudo -E` or you won't see the logs.

Then in another terminal, run commands:
```bash
ls /tmp
curl http://example.com
python3 --version
cat /etc/hosts
```

Watch the first terminal catch every process launch in real-time. This is how security tools like Falco detect suspicious activity.

## Key Differences

| Feature | Packet Monitor | Process Tracker |
|---------|---------------|-----------------|
| **eBPF Type** | TC Classifier | Tracepoint |
| **Domain** | Networking | Observability |
| **Hook** | Network stack | Scheduler |
| **Stability** | sk_buff API | Stable tracepoint API |
| **Coolness** | Basic | Actually impressive |

## Why Two Different Demos?

To show eBPF's versatility:
- Not just for networking
- Same Aya framework works for everything
- Can combine different program types (like my thesis does)

My lockne project uses both TC classifiers (for packet inspection) and cgroup programs (for process tracking) working together. These demos show simplified versions of each approach.

## Building Both

From this directory:
```bash
for demo in 01-packet-monitor 02-process-tracker; do
    (cd $demo && cargo build)
done
```

Or just build them individually as needed.
