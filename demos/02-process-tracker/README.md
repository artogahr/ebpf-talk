# Process Tracker

eBPF tracepoint that watches for new process execution.

Attaches to the `sched/sched_process_exec` tracepoint to catch every time a new process starts.

## What It Does

Every time you run a command (`ls`, `curl`, `python`, etc), the kernel calls the scheduler's exec tracepoint. Our eBPF program hooks into that and logs:
- Process name
- PID

This is basically a real-time `ps` that shows process launches as they happen.

## Building

```bash
cargo build
```

Requires the same toolchain as the packet monitor (bpf-linker, rustup nightly, etc).

## Running

**Important:** You need `RUST_LOG=info` to see the logs!

```bash
RUST_LOG=info sudo -E ./target/debug/file-tracker --duration 30
```

The `-E` flag preserves the `RUST_LOG` environment variable through sudo.

Options:
- `--duration <secs>` - Auto-exit after N seconds (default: run until Ctrl-C)

### Testing

Start the tracker in one terminal:
```bash
RUST_LOG=info sudo -E ./target/debug/file-tracker --duration 30
```

In another terminal, run various commands:
```bash
ls /tmp
curl http://example.com
python3 --version
cat /etc/hosts
vim test.txt
```

You'll see output like:
```
[INFO  file_tracker] 🚀 Process started: ls (PID 377673)
[INFO  file_tracker] 🚀 Process started: curl (PID 377674)
[INFO  file_tracker] 🚀 Process started: python3 (PID 377675)
```

## How It Works

**Kernel side** (`file-tracker-ebpf/src/main.rs`):
- `#[tracepoint]` macro marks this as a tracepoint program
- Attaches to `sched/sched_process_exec` 
- Extracts PID using `bpf_get_current_pid_tgid()` helper
- Extracts process name using `bpf_get_current_comm()` helper
- Logs both to userspace

**User side** (`file-tracker/src/main.rs`):
- Loads compiled eBPF bytecode
- Attaches to `sched/sched_process_exec` tracepoint
- Sets up async logger to receive kernel logs
- Waits for Ctrl-C or timeout

## Why This Is Interesting

Unlike the packet monitor, this has nothing to do with networking. It's pure system observability.

**Real-world uses:**
- Security monitoring (Falco does this to detect suspicious processes)
- Debugging ("what the hell is spawning all these processes?")
- Auditing and compliance
- Container runtime monitoring

Tracepoints are also more stable than kprobes - they're part of the kernel's stable API, so they won't break between kernel versions.

## Comparison to Lockne

My thesis project uses cgroup programs for process tracking, which is a different approach:
- **This demo:** Tracepoint on process exec (catches all processes)
- **Lockne:** Cgroup hook on socket operations (only tracks network-related processes)

Both are valid approaches depending on what you need. Tracepoints are simpler but give you everything. Cgroup hooks are more targeted.
