#![no_std]
#![no_main]

use aya_ebpf::{macros::tracepoint, programs::TracePointContext};
use aya_log_ebpf::info;

#[tracepoint]
pub fn file_tracker(ctx: TracePointContext) -> u32 {
    match try_file_tracker(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_file_tracker(ctx: TracePointContext) -> Result<u32, u32> {
    // Get PID (upper 32 bits of pid_tgid)
    let pid = (unsafe { aya_ebpf::helpers::bpf_get_current_pid_tgid() } >> 32) as u32;
    
    // Get process name (comm) - new API returns Result<[u8; 16]>
    let comm = aya_ebpf::helpers::bpf_get_current_comm()
        .unwrap_or([0u8; 16]);
    
    // Convert comm to string for logging
    let comm_str = unsafe {
        let mut len = 0;
        for &byte in comm.iter() {
            if byte == 0 {
                break;
            }
            len += 1;
        }
        core::str::from_utf8_unchecked(&comm[..len])
    };
    
    info!(&ctx, "🚀 Process started: {} (PID {})", comm_str, pid);
    Ok(0)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(link_section = "license")]
#[unsafe(no_mangle)]
static LICENSE: [u8; 13] = *b"Dual MIT/GPL\0";
