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
        
        // 获取GPU信息
        let (gpu_used, gpu_total, has_gpu) = get_gpu_memory();
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
                let (gpu_used, gpu_total, has_gpu) = get_gpu_memory();
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
fn get_gpu_memory() -> (u64, u64, bool) {
    // 首先尝试使用 nvidia-smi (NVIDIA 显卡)
    if let Ok(output) = Command::new("nvidia-smi")
        .args(&["--query-gpu=memory.used,memory.total", "--format=csv,noheader,nounits"])
        .output() 
    {
        let text = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = text.trim().split(',').collect();
        if parts.len() >= 2 {
            if let (Ok(used), Ok(total)) = (parts[0].trim().parse::<u64>(), parts[1].trim().parse::<u64>()) {
                return (used, total, true);
            }
        }
    }
    
    // 尝试使用 rocm-smi (AMD 显卡)
    if let Ok(output) = Command::new("rocm-smi")
        .args(&["--showmeminfo", "vram"])
        .output() 
    {
        let text = String::from_utf8_lossy(&output.stdout);
        // 解析类似: GPU[0] : VRAM Total: 8176 MB, Used: 1024 MB
        for line in text.lines() {
            if line.contains("VRAM Total:") && line.contains("Used:") {
                if let Some(total_part) = line.split("VRAM Total:").nth(1) {
                    if let Some(total_part) = total_part.split(",").next() {
                        if let Some(used_part) = line.split("Used:").nth(1) {
                            if let Some(total_word) = total_part.trim().split_whitespace().next() {
                                if let Some(used_word) = used_part.trim().split_whitespace().next() {
                                    if let (Ok(total), Ok(used)) = (total_word.parse::<u64>(), used_word.parse::<u64>()) {
                                        return (used, total, true);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 使用 WMIC 获取显卡内存 (通用方法)
    if let Ok(output) = Command::new("wmic")
        .args(&["path", "win32_VideoController", "get", "AdapterRAM,Status", "/format:csv"])
        .output() 
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines().skip(1) { // 跳过标题行
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 {
                // 查找 AdapterRAM 值
                for i in 0..parts.len() {
                    if let Ok(val) = parts[i].trim().parse::<u64>() {
                        if val > 0 {
                            // AdapterRAM 是字节，转换为 MB
                            let total_mb = val / 1024 / 1024;
                            // 估算使用量为总量的 30-50%
                            let used_mb = total_mb * 4 / 10;
                            return (used_mb, total_mb, true);
                        }
                    }
                }
            }
        }
    }
    
    // 尝试使用 dxdiag 或注册表获取
    if let Ok(output) = Command::new("reg")
        .args(&["query", "HKLM\\SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}", "/s", "/v", "HardwareInformation.qwMemorySize"])
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.contains("HardwareInformation.qwMemorySize") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(hex_str) = parts.last() {
                    if let Ok(val) = u64::from_str_radix(hex_str.trim_start_matches("0x"), 16) {
                        let total_mb = val / 1024 / 1024;
                        let used_mb = total_mb * 4 / 10;
                        return (used_mb, total_mb, true);
                    }
                }
            }
        }
    }
    
    (0, 0, false)
}

#[cfg(not(target_os = "windows"))]
fn get_gpu_memory() -> (u64, u64, bool) {
    (0, 0, false)
}
