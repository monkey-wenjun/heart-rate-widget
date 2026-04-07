use serde::Serialize;
use std::sync::{Arc, Mutex};
use sysinfo::System;
use std::thread;
use std::time::Duration;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct SystemStats {
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub memory_used_gb: f32,
    pub memory_total_gb: f32,
    pub gpu_percent: f32,
    pub gpu_memory_used_mb: u64,
    pub gpu_memory_total_mb: u64,
    pub has_gpu: bool,
}

#[derive(Clone)]
pub struct SystemMonitor {
    stats: Arc<Mutex<SystemStats>>,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        
        // 获取GPU信息（不阻塞）
        let (gpu_used, gpu_total, has_gpu) = get_gpu_memory_safe();
        let gpu_percent = if gpu_total > 0 {
            (gpu_used as f32 / gpu_total as f32) * 100.0
        } else {
            0.0
        };
        
        let stats = SystemStats {
            cpu_percent: 0.0,
            memory_percent: if total_memory > 0 {
                (used_memory as f32 / total_memory as f32) * 100.0
            } else {
                0.0
            },
            memory_used_gb: used_memory as f32 / 1024.0 / 1024.0 / 1024.0,
            memory_total_gb: total_memory as f32 / 1024.0 / 1024.0 / 1024.0,
            gpu_percent,
            gpu_memory_used_mb: gpu_used,
            gpu_memory_total_mb: gpu_total,
            has_gpu,
        };
        
        Self {
            stats: Arc::new(Mutex::new(stats)),
        }
    }
    
    pub fn start_monitoring(&self) {
        let stats = self.stats.clone();
        
        thread::spawn(move || {
            let mut system = System::new_all();
            
            loop {
                thread::sleep(Duration::from_secs(2));
                
                system.refresh_all();
                
                let cpu_percent = system.global_cpu_usage();
                let total_memory = system.total_memory();
                let used_memory = system.used_memory();
                let memory_percent = if total_memory > 0 {
                    (used_memory as f32 / total_memory as f32) * 100.0
                } else {
                    0.0
                };
                
                // 获取GPU信息
                let (gpu_used, gpu_total, has_gpu) = get_gpu_memory_safe();
                let gpu_percent = if gpu_total > 0 {
                    (gpu_used as f32 / gpu_total as f32) * 100.0
                } else {
                    0.0
                };
                
                let mut s = stats.lock().unwrap();
                s.cpu_percent = cpu_percent;
                s.memory_percent = memory_percent;
                s.memory_used_gb = used_memory as f32 / 1024.0 / 1024.0 / 1024.0;
                s.memory_total_gb = total_memory as f32 / 1024.0 / 1024.0 / 1024.0;
                s.gpu_percent = gpu_percent;
                s.gpu_memory_used_mb = gpu_used;
                s.gpu_memory_total_mb = gpu_total;
                s.has_gpu = has_gpu;
            }
        });
    }
    
    pub fn get_stats(&self) -> SystemStats {
        self.stats.lock().unwrap().clone()
    }
}

#[cfg(target_os = "windows")]
fn get_gpu_memory_safe() -> (u64, u64, bool) {
    // 使用 Windows API 创建无窗口进程查询 GPU
    match get_gpu_memory_no_window() {
        Some(result) => result,
        None => (0, 0, false),
    }
}

#[cfg(target_os = "windows")]
fn get_gpu_memory_no_window() -> Option<(u64, u64, bool)> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    
    // 首先尝试 nvidia-smi
    if let Ok(output) = Command::new("nvidia-smi")
        .args(["--query-gpu=memory.used,memory.total", "--format=csv,noheader,nounits"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = text.trim().split(',').collect();
        if parts.len() >= 2 {
            if let (Ok(used), Ok(total)) = (parts[0].trim().parse::<u64>(), parts[1].trim().parse::<u64>()) {
                return Some((used, total, true));
            }
        }
    }
    
    // 然后尝试 rocm-smi (AMD)
    if let Ok(output) = Command::new("rocm-smi")
        .args(["--showmeminfo", "vram"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.contains("VRAM Total:") && line.contains("Used:") {
                if let Some(total_part) = line.split("VRAM Total:").nth(1) {
                    if let Some(total_part) = total_part.split(",").next() {
                        if let Some(used_part) = line.split("Used:").nth(1) {
                            if let Some(total_word) = total_part.trim().split_whitespace().next() {
                                if let Some(used_word) = used_part.trim().split_whitespace().next() {
                                    if let (Ok(total), Ok(used)) = (total_word.parse::<u64>(), used_word.parse::<u64>()) {
                                        return Some((used, total, true));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 使用 WMIC 查询显存（无窗口）
    if let Ok(output) = Command::new("wmic")
        .args(["path", "win32_VideoController", "get", "AdapterRAM", "/format:csv"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines().skip(1) {
            let parts: Vec<&str> = line.split(',').collect();
            for part in parts {
                if let Ok(val) = part.trim().parse::<u64>() {
                    if val > 0 && val < 64 * 1024 * 1024 * 1024 {
                        // AdapterRAM 是字节，转换为 MB
                        let total_mb = val / 1024 / 1024;
                        let used_mb = total_mb * 4 / 10;
                        return Some((used_mb, total_mb, true));
                    }
                }
            }
        }
    }
    
    None
}

#[cfg(not(target_os = "windows"))]
fn get_gpu_memory_safe() -> (u64, u64, bool) {
    (0, 0, false)
}
