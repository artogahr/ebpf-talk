---
title: eBPF, Aya and Network Programming using Rust
author: Arto Gahr
theme: 
    name: terminal-dark
---

<!-- end_slide -->

# eBPF, Aya and Network Programming using Rust

**Or: How I learned to stop worrying and love the kernel** 🚀

Artoghrul Gahramanli (Arto)

Master's Thesis @ CZU Prague

<!-- end_slide -->

# whoami

**Arto Gahr** - Systems programming enthusiast

📚 **Thesis:** "Lockne: Dynamic Per-Application VPN Tunneling with eBPF and Rust"

**The problem:** Route Firefox through VPN, but keep games on direct connection

*(Yes, I'm voluntarily debugging kernel code. No, therapy isn't helping.)*

<!-- end_slide -->

# What is eBPF?

**e**xtended **B**erkeley **P**acket **F**ilter

*"It's like JavaScript for the Linux kernel!"* - Someone, probably

**Really though:**
- Run **sandboxed programs** in the **kernel**
- Without writing kernel modules
- Without rebooting
- Without (usually) crashing everything 💥

<!-- end_slide -->

# The Old Way vs The eBPF Way

**Old Way:** Kernel Module

```c
// 1. Write module for exact kernel version
// 2. insmod your_module.ko
// 3. Kernel panic
// 4. Reboot server
// 5. Get fired
```

**eBPF Way:**

```rust
// 1. Write once
// 2. bpf(BPF_PROG_LOAD, ...)
// 3. It works! (or verifier says no)
// 4. Keep your job
```

*\*Famous last words: "The verifier will catch any bugs!"*

<!-- end_slide -->

# Why eBPF is Cool

**🚀 Speed**
- Runs in kernel space
- **~60ns overhead per packet** (measured in my thesis!)
- JIT compiled to native code

**🔒 Safety**
- Verified before loading
- Bounded loops only
- Can't crash kernel*

**🎯 Flexibility**
- Load/unload dynamically
- No reboot needed
- Real-time updates

*\*The verifier is very strict. Like, VERY strict.*

<!-- end_slide -->

# eBPF Program Types

**XDP** - eXpress Data Path (fastest, at driver level)
**TC** - Traffic Control (my thesis uses this!)
**Kprobes** - Trace kernel functions
**Uprobes** - Trace userspace functions  
**Cgroup programs** - Per-process hooks (also in my thesis!)
**Tracepoints** - Stable kernel events

*It's like Pokémon but for observability* 🎮

<!-- end_slide -->

# Real-World eBPF

**Networking**
- Cilium (Kubernetes CNI - used everywhere)
- Katran (Facebook's L4 load balancer)

**Observability**
- bpftrace (like DTrace for Linux)
- Pixie (Kubernetes observability)

**Security**
- Falco (runtime threat detection)
- Tetragon (eBPF-based security)

*If it touches the kernel, someone made an eBPF tool for it*

<!-- end_slide -->

# Why Rust for eBPF?

**C/BCC Approach:**

```c
#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, __u64);
    __type(value, __u32);
    __uint(max_entries, 10240);
} socket_map SEC(".maps");

// Manual type casts everywhere
// Seg faults if you mess up
// No compile-time checks
```

<!-- end_slide -->

# Why Rust for eBPF? (continued)

**Aya/Rust Approach:**

```rust
#[map]
static SOCKET_MAP: HashMap<u64, u32> = 
    HashMap::with_max_entries(10240, 0);

// Type-safe!
// Compiler catches errors
// Much better DX
```

**Benefits:**
✅ Memory safety ✅ Modern tooling
✅ Same performance ✅ Better errors

<!-- end_slide -->

# Meet Aya 🎯

**Pure Rust eBPF library** (no LLVM/Clang needed*)

```rust
use aya::{Ebpf, programs::*};

// Load eBPF program
let mut bpf = Ebpf::load(MY_PROGRAM)?;

// Get and attach TC classifier
let prog: &mut SchedClassifier = 
    bpf.program_mut("demo").try_into()?;
prog.load()?;
prog.attach("eth0", TcAttachType::Egress)?;

// That's it! Running in kernel now!
```

*\*You still need bpf-linker, but it's just `cargo install`*

<!-- end_slide -->

# Aya Architecture

**Kernel Space** (`aya-ebpf` crate)
- `#![no_std]` environment
- eBPF programs as functions
- Access to kernel helpers
- Compiled to BPF bytecode

**User Space** (`aya` crate)
- Normal Rust with std
- Loads & manages programs
- Reads/writes eBPF maps
- Handles logging

**Shared** (`your-common` crate)
- Types used by both sides
- Must be `#![no_std]`

<!-- end_slide -->

# eBPF Program Types - Real Examples

**Different hooks for different jobs:**

**TC Classifier** (like today's demo)
```rust
#[classifier]  // Packet processing
```

**XDP** (even faster!)
```rust
#[xdp]  // At driver level, for DDoS protection
```

**Tracepoint** (system observability)
```rust
#[tracepoint]  // Track syscalls, file opens
```

**Cgroup** (per-process hooks)
```rust  
#[cgroup_sock_addr]  // My thesis uses this!
```

**My lockne uses TWO types: TC + Cgroup working together!**

<!-- end_slide -->

# My Thesis: lockne

**Problem:** Per-application VPN routing

**Traditional VPN:** All-or-nothing
- Enable VPN → Everything goes through tunnel
- Slow games, high latency, can't access local network

**What I wanted:**
- Firefox → VPN (privacy)
- Games → Direct (low latency)
- Work apps → Corporate VPN

**Existing solutions:** Proxies (slow), containers (complex)

<!-- end_slide -->

# How lockne Works

**Architecture:**

```
Application makes connection
         ↓
Cgroup/sock_addr hook captures:
  - Socket cookie (unique ID)
  - Process ID (PID)
         ↓
Store in eBPF map: cookie → PID
         ↓
Packet sent → TC egress hook
         ↓
Extract socket cookie from packet
         ↓
Look up PID in map → Found it!
```

**Two programs working together!**

<!-- end_slide -->

# The Socket Cookie Trick

**Problem:** Packets don't have PID info

**Solution:** Socket cookies!

```rust
// In cgroup program (when socket created):
let cookie = bpf_get_socket_cookie(ctx.sock_addr);
let pid = bpf_get_current_pid_tgid() >> 32; // Upper 32 bits
SOCKET_MAP.insert(&cookie, &pid, 0)?;

// In TC program (when packet sent):
let cookie = bpf_get_socket_cookie(ctx.skb.skb);
if let Some(pid) = SOCKET_MAP.get(&cookie) {
    // We know which process sent this!
}
```

**Key insight:** Socket cookies are stable identifiers

<!-- end_slide -->

# Code Comparison: Maps

**C/BCC:**

```c
BPF_HASH(socket_map, u64, u32, 10240);

// Accessing the map:
u32 *pid = socket_map.lookup(&cookie);
if (pid) {
    // Use pid
} else {
    // Handle missing
}
// Easy to mess up pointer checks
```

<!-- end_slide -->

# Code Comparison: Maps (continued)

**Aya/Rust:**

```rust
#[map]
static SOCKET_MAP: HashMap<u64, u32> = 
    HashMap::with_max_entries(10240, 0);

// Accessing the map:
if let Some(pid) = unsafe { SOCKET_MAP.get(&cookie) } {
    // pid is &u32, can't mess up
    info!(&ctx, "pid={}", *pid);
}
// Compiler enforces safety
```

**`unsafe` is explicit, types are checked!**

<!-- end_slide -->

# Real Challenges I Hit

**1. Testing with `ping` didn't work**
- ICMP doesn't call `connect()`
- Cgroup hook never fired
- Switched to `curl` (TCP) ✅

**2. Pre-existing connections show "unknown"**
- Can only track NEW connections
- Solution: Launch apps through lockne
- `sudo lockne run firefox`

**3. The verifier hates me**
- "Cannot prove bounded execution"
- "Invalid memory access"
- Learned to think like the verifier 🤖

<!-- end_slide -->

# What Actually Works

**✅ Process tracking via socket cookies**
- Reliably maps packets → PIDs

**✅ Two-program architecture**
- TC classifier + cgroup tracker

**✅ Process launcher mode**
- `lockne run <program>` just works

**✅ Performance**
- ~60ns per packet overhead
- <1% CPU usage
- 200KB memory

**✅ TUI with ratatui**
- Live stats, process counts

<!-- end_slide -->

# What's Still TODO

**❌ Actual packet redirection**
- Currently just tracking
- Need `bpf_redirect()` to WireGuard interface

**❌ IPv6 support**
- Only IPv4 right now

**❌ Map cleanup**
- Entries never removed
- Need to track socket close

**❌ Process hierarchy**
- Child processes not tracked

*These are clearly defined next steps!*

<!-- end_slide -->

# Code Comparison: Program Structure

**C/BCC:**

```c
SEC("classifier")
int tc_classifier(struct __sk_buff *skb) {
    // Manual pointer arithmetic
    void *data = (void *)(long)skb->data;
    void *data_end = (void *)(long)skb->data_end;
    
    struct ethhdr *eth = data;
    if ((void *)(eth + 1) > data_end)
        return TC_ACT_OK; // Bounds check
    
    // More manual checks...
}
```

<!-- end_slide -->

# Code Comparison: Program Structure (continued)

**Aya/Rust:**

```rust
#[classifier]
pub fn tc_classifier(ctx: TcContext) -> i32 {
    match try_classify(ctx) {
        Ok(ret) => ret,
        Err(_) => TC_ACT_PIPE,
    }
}

fn try_classify(ctx: TcContext) -> Result<i32, ()> {
    let eth: EthHdr = ctx.load(0)?; // Bounds checked!
    let ip: Ipv4Hdr = ctx.load(ETH_HLEN)?; // Type safe!
    // ...
}
```

**The verifier still checks everything, but Rust helps you write correct code!**

<!-- end_slide -->

# Demo Time! 🎬

**Simple packet monitor** (official Aya template)

What you'll see:
- eBPF program attaching to interface
- Logs for every packet
- All in Rust!
- Exits after N seconds (not hanging!)

```bash
sudo ./target/debug/demo --iface lo --duration 10
```

*What could possibly go wrong?* 😅

<!-- end_slide -->

# Lessons Learned

**1. Start with Aya template**
- Don't build from scratch
- `cargo generate` saves weeks

**2. Test incrementally**
- Add one feature at a time
- eBPF debugging is HARD

**3. Read the verifier errors**
- They're cryptic but correct
- "Cannot prove bounded" = you need a loop limit

**4. C experience helps**
- Understanding packet structures
- Knowing kernel concepts

**5. Rust makes it better**
- Catches bugs at compile time
- Better refactoring

<!-- end_slide -->

# Performance: eBPF vs Userspace

**My measurements (from thesis):**

|Approach|Per-Packet|CPU|Memory|
|--------|----------|---|------|
|**Lockne (eBPF)**|~60ns|<1%|200KB|
|Userspace Proxy|~10-50µs|5-10%|10-50MB|

**eBPF is 100-1000x faster!**

Why?
- No context switching
- No copying to userspace
- Kernel-native data structures

<!-- end_slide -->

# Resources 📚

**eBPF**
- [ebpf.io](https://ebpf.io) - Start here!
- [ebpf.io/what-is-ebpf](https://ebpf.io/what-is-ebpf) - Great intro

**Aya**
- [aya-rs.dev](https://aya-rs.dev) - Official docs
- [github.com/aya-rs/aya](https://github.com/aya-rs/aya) - Source
- [github.com/aya-rs/aya-template](https://github.com/aya-rs/aya-template) - Start here!

**My Project**
- Ask me about lockne!
- Code is on my machine (maybe GitHub soon?)

<!-- end_slide -->

# Questions? 💬

*"I don't know the answer, but the verifier probably does!"* 😅

Happy to chat about:
- eBPF quirks and footguns
- Rust for systems programming
- My thesis struggles
- Why socket cookies are amazing
- How the verifier ruined my weekend

**Thank you!** 🎉

<!-- end_slide -->

# Backup: Technical Details

**Socket Cookie Extraction:**

```rust
// Different contexts need different pointers!

// In cgroup/sock_addr:
let cookie = bpf_get_socket_cookie(ctx.sock_addr as *mut _);

// In TC classifier:
let cookie = bpf_get_socket_cookie(ctx.skb.skb as *mut _);

// Took me hours to figure this out...
```

**PID extraction:**

```rust
let pid_tgid = bpf_get_current_pid_tgid();
let pid = (pid_tgid >> 32) as u32;  // Upper 32 bits = TGID
let tid = (pid_tgid & 0xFFFFFFFF) as u32;  // Lower 32 = TID
```
