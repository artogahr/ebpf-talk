use aya::programs::TracePoint;
use clap::Parser;
#[rustfmt::skip]
use log::{debug, warn};
use tokio::signal;

#[derive(Debug, Parser)]
struct Opt {
    /// Exit after N seconds (useful for testing)
    #[clap(long)]
    duration: Option<u64>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    
    env_logger::init();

    // bump memlock rlimit for older kernels
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        debug!("remove limit on locked memory failed, ret is: {ret}");
    }

    // load eBPF program
    let mut ebpf = aya::Ebpf::load(aya::include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/file-tracker"
    )))?;
    match aya_log::EbpfLogger::init(&mut ebpf) {
        Err(e) => {
            warn!("failed to initialize eBPF logger: {e}");
        }
        Ok(logger) => {
            let mut logger =
                tokio::io::unix::AsyncFd::with_interest(logger, tokio::io::Interest::READABLE)?;
            tokio::task::spawn(async move {
                loop {
                    let mut guard = logger.readable_mut().await.unwrap();
                    guard.get_inner_mut().flush();
                    guard.clear_ready();
                }
            });
        }
    }
    let program: &mut TracePoint = ebpf.program_mut("file_tracker").unwrap().try_into()?;
    program.load()?;
    program.attach("sched", "sched_process_exec")?;

    println!("🔍 Process Tracker started!");
    println!("Watching for new processes being launched...");
    println!("Try: ls, curl, python, etc. in another terminal");
    println!();

    if let Some(dur) = opt.duration {
        println!("Running for {} seconds...", dur);
        tokio::time::sleep(tokio::time::Duration::from_secs(dur)).await;
        println!("Duration elapsed, exiting...");
    } else {
        println!("Press Ctrl-C to exit");
        signal::ctrl_c().await?;
        println!("Exiting...");
    }

    Ok(())
}
