//! Info command - display system information for bug reports

use anyhow::Result;
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn run() -> Result<()> {
    println!("Bounty Challenge System Information");
    println!("====================================");
    println!();
    println!("Version: {}", VERSION);
    println!();

    // OS Information
    println!("## Operating System");
    print_os_info();
    println!();

    // Hardware Information
    println!("## Hardware");
    print_hardware_info();
    println!();

    // Rust/Cargo version
    println!("## Build Environment");
    print_build_info();
    println!();

    // Environment variables (non-sensitive)
    println!("## Environment");
    print_env_info();

    println!();
    println!("====================================");
    println!("Copy the above output for bug reports");

    Ok(())
}

fn print_os_info() {
    // Try to get OS info
    #[cfg(target_os = "linux")]
    {
        // Try /etc/os-release first
        if let Ok(output) = std::fs::read_to_string("/etc/os-release") {
            for line in output.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    let name = line.trim_start_matches("PRETTY_NAME=").trim_matches('"');
                    println!("  OS: {}", name);
                    break;
                }
            }
        }

        // Kernel version
        if let Ok(output) = Command::new("uname").args(["-r"]).output() {
            if output.status.success() {
                let kernel = String::from_utf8_lossy(&output.stdout);
                println!("  Kernel: {}", kernel.trim());
            }
        }

        // Architecture
        if let Ok(output) = Command::new("uname").args(["-m"]).output() {
            if output.status.success() {
                let arch = String::from_utf8_lossy(&output.stdout);
                println!("  Arch: {}", arch.trim());
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("sw_vers").output() {
            if output.status.success() {
                let info = String::from_utf8_lossy(&output.stdout);
                for line in info.lines() {
                    println!("  {}", line);
                }
            }
        }

        if let Ok(output) = Command::new("uname").args(["-m"]).output() {
            if output.status.success() {
                let arch = String::from_utf8_lossy(&output.stdout);
                println!("  Arch: {}", arch.trim());
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        println!("  OS: Windows");
        if let Ok(output) = Command::new("cmd").args(["/C", "ver"]).output() {
            if output.status.success() {
                let ver = String::from_utf8_lossy(&output.stdout);
                println!("  Version: {}", ver.trim());
            }
        }
    }
}

fn print_hardware_info() {
    #[cfg(target_os = "linux")]
    {
        // CPU info
        if let Ok(output) = std::fs::read_to_string("/proc/cpuinfo") {
            let mut model_name = None;
            let mut cpu_cores = 0;

            for line in output.lines() {
                if line.starts_with("model name") && model_name.is_none() {
                    model_name = line.split(':').nth(1).map(|s| s.trim().to_string());
                }
                if line.starts_with("processor") {
                    cpu_cores += 1;
                }
            }

            if let Some(model) = model_name {
                println!("  CPU: {} ({} cores)", model, cpu_cores);
            }
        }

        // Memory info
        if let Ok(output) = std::fs::read_to_string("/proc/meminfo") {
            for line in output.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            let gb = kb / 1024 / 1024;
                            println!("  RAM: {} GB", gb);
                        }
                    }
                    break;
                }
            }
        }

        // GPU info (if nvidia-smi available)
        if let Ok(output) = Command::new("nvidia-smi")
            .args(["--query-gpu=name,memory.total", "--format=csv,noheader"])
            .output()
        {
            if output.status.success() {
                let gpu_info = String::from_utf8_lossy(&output.stdout);
                for (i, line) in gpu_info.lines().enumerate() {
                    println!("  GPU {}: {}", i, line.trim());
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
        {
            if output.status.success() {
                let cpu = String::from_utf8_lossy(&output.stdout);
                println!("  CPU: {}", cpu.trim());
            }
        }

        if let Ok(output) = Command::new("sysctl").args(["-n", "hw.memsize"]).output() {
            if output.status.success() {
                let bytes_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(bytes) = bytes_str.trim().parse::<u64>() {
                    let gb = bytes / 1024 / 1024 / 1024;
                    println!("  RAM: {} GB", gb);
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        println!("  (Run 'systeminfo' for detailed hardware info)");
    }
}

fn print_build_info() {
    println!("  Rust: {}", rustc_version());
    println!("  Target: {}", std::env::consts::ARCH);
}

fn rustc_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn print_env_info() {
    // Only print non-sensitive env vars
    let safe_vars = [
        "RUST_LOG",
        "CHALLENGE_HOST",
        "CHALLENGE_PORT",
        "PLATFORM_URL",
    ];

    for var in safe_vars {
        if let Ok(val) = std::env::var(var) {
            println!("  {}: {}", var, val);
        }
    }

    // Check if DATABASE_URL is set (but don't print the value)
    if std::env::var("DATABASE_URL").is_ok() {
        println!("  DATABASE_URL: [set]");
    }
}
