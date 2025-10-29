# Talk Notes

~25 min total

## Structure

- slides (12-15 min)
- process tracker demo (3 min) 
- live code ping thing (8-10 min)
- q&a

## Running Stuff

### Slides
```bash
cd slides
presenterm presentation.md
```

### Process tracker demo (pre-built)

Terminal 1:
```bash
cd demos/02-process-tracker
RUST_LOG=info sudo -E ./target/debug/file-tracker --duration 60
```

Terminal 2 - run some stuff:
```bash
ls
curl http://example.com
python3 --version
```

Talk about how it's a tracepoint, not TC. different use case (observability vs networking).

## Live Demo - Ping Monitor

my laptop ip (check before talk):
```bash
ip addr show eno1 | grep "inet "
# should be 10.0.0.70
```

### Setup

```bash
cd ~/workplace/ebpf-talk
cp aya-env-template.nix flake.nix
git add flake.nix
nix develop
```

### Generate

```bash
cargo generate --name ping-monitor \
  -d program_type=classifier \
  -d direction=Ingress \
  -d default_iface=eno1 \
  https://github.com/aya-rs/aya-template
```

### Code to write

edit `ping-monitor-ebpf/src/main.rs`

replace try_ping_monitor:

```rust
fn try_ping_monitor(ctx: TcContext) -> Result<i32, i32> {
    let ethertype: u16 = unsafe { ptr_at(&ctx, 12)? };
    if ethertype != 0x0008 {  // IPv4
        return Ok(TC_ACT_PIPE);
    }

    let protocol: u8 = unsafe { ptr_at(&ctx, 23)? };
    if protocol != 1 {  // ICMP
        return Ok(TC_ACT_PIPE);
    }

    let icmp_type: u8 = unsafe { ptr_at(&ctx, 34)? };
    
    if icmp_type == 8 {  // Echo Request
        info!(&ctx, "🔔 Someone is pinging us!");
    }

    Ok(TC_ACT_PIPE)
}
```

add helper before panic_handler:

```rust
#[inline(always)]
unsafe fn ptr_at<T: Copy>(ctx: &TcContext, offset: usize) -> Result<T, i32> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = core::mem::size_of::<T>();
    
    if start + offset + len > end {
        return Err(TC_ACT_PIPE);
    }
    
    Ok(*((start + offset) as *const T))
}
```

### Run

```bash
cd ping-monitor
RUST_LOG=info cargo run --config 'target."cfg(all())".runner="sudo -E"' -- --iface eno1
```

### Demo from proxmox

ssh to proxmox in another terminal, then:
```bash
ping 10.0.0.70
```

should see logs appear.

## Numbers to remember

- 0x0008 = IPv4 
- 1 = ICMP
- 8 = Echo Request (ping)
- offsets: 12 (ethertype), 23 (protocol), 34 (icmp type)

## Points to mention

- parsing ethernet -> ip -> icmp
- type 8 is incoming ping request
- verifier needs bounds checking (that's the ptr_at helper)
- this is what monitoring tools like cilium/falco do
- built in ~10 min

## If stuff breaks

- just skip to q&a, slides + process tracker are fine on their own
- or show the code and explain without running

## Test before talk

run through it once the night before to make sure:
- know laptop ip
- ssh to proxmox works  
- ping from proxmox reaches laptop
- logs appear

that's it
