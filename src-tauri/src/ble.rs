use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

// 心率服务 UUID
const HEART_RATE_SERVICE_UUID: Uuid = Uuid::from_u128(0x0000180d_0000_1000_8000_00805f9b34fb);
// 心率测量特征 UUID
const HEART_RATE_MEASUREMENT_UUID: Uuid = Uuid::from_u128(0x00002a37_0000_1000_8000_00805f9b34fb);

#[derive(Debug, Clone, serde::Serialize)]
pub struct HeartRateData {
    pub heart_rate: u8,
    pub sensor_contact: bool,
    pub energy_expended: Option<u16>,
    pub rr_intervals: Vec<u16>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BleDevice {
    pub id: String,
    pub name: String,
    pub rssi: Option<i16>,
}

/// 解析心率测量数据
pub fn parse_heart_rate_data(data: &[u8]) -> Option<HeartRateData> {
    eprintln!("收到原始数据: {:?}", data);
    
    if data.is_empty() {
        eprintln!("数据为空");
        return None;
    }

    let flags = data[0];
    let heart_rate_format = flags & 0x01; // 0 = uint8, 1 = uint16
    let sensor_contact_status = (flags >> 1) & 0x03;
    let energy_expended_present = (flags >> 3) & 0x01;
    let rr_interval_present = (flags >> 4) & 0x01;

    eprintln!("Flags: {:08b}, HR格式: {}, 传感器: {}, 能量: {}, RR: {}", 
              flags, heart_rate_format, sensor_contact_status, energy_expended_present, rr_interval_present);

    let mut index = 1;

    // 解析心率值
    let heart_rate = if heart_rate_format == 0 {
        if data.len() > index {
            let hr = data[index];
            index += 1;
            hr
        } else {
            eprintln!("数据长度不足(uint8)");
            return None;
        }
    } else {
        if data.len() > index + 1 {
            let hr = u16::from_le_bytes([data[index], data[index + 1]]) as u8;
            index += 2;
            hr
        } else {
            eprintln!("数据长度不足(uint16)");
            return None;
        }
    };

    let sensor_contact = sensor_contact_status == 2 || sensor_contact_status == 3;

    // 能量消耗
    let energy_expended = if energy_expended_present == 1 {
        if data.len() > index + 1 {
            let ee = u16::from_le_bytes([data[index], data[index + 1]]);
            index += 2;
            Some(ee)
        } else {
            None
        }
    } else {
        None
    };

    // RR 间隔
    let mut rr_intervals = Vec::new();
    if rr_interval_present == 1 {
        while data.len() >= index + 2 {
            let rr = u16::from_le_bytes([data[index], data[index + 1]]);
            rr_intervals.push(rr);
            index += 2;
        }
    }

    eprintln!("解析成功: 心率={}, 传感器={}", heart_rate, sensor_contact);

    Some(HeartRateData {
        heart_rate,
        sensor_contact,
        energy_expended,
        rr_intervals,
    })
}

/// 扫描心率设备
pub async fn scan_heart_rate_devices(duration_secs: u64) -> Result<Vec<BleDevice>, String> {
    eprintln!("开始扫描蓝牙设备...");
    
    let manager = Manager::new().await.map_err(|e| format!("创建蓝牙管理器失败: {}", e))?;
    
    let adapters = manager.adapters().await.map_err(|e| format!("获取适配器失败: {}", e))?;
    
    if adapters.is_empty() {
        return Err("未找到蓝牙适配器".to_string());
    }

    let central = adapters.into_iter().next().unwrap();
    eprintln!("使用适配器扫描...");

    central
        .start_scan(ScanFilter::default())
        .await
        .map_err(|e| format!("开始扫描失败: {}", e))?;

    eprintln!("扫描中... 等待 {} 秒", duration_secs);
    time::sleep(Duration::from_secs(duration_secs)).await;

    let peripherals = central.peripherals().await.map_err(|e| format!("获取设备失败: {}", e))?;
    eprintln!("发现 {} 个设备", peripherals.len());
    
    let mut devices = Vec::new();

    for p in peripherals {
        if let Ok(Some(props)) = p.properties().await {
            let name = props.local_name.unwrap_or_else(|| "未知设备".to_string());
            
            eprintln!("设备: {} | 服务: {:?}", name, props.services);
            
            // 支持的心率设备
            let name_lower = name.to_lowercase();
            let is_hr_device = 
                name_lower.contains("polar") 
                || name_lower.contains("garmin")
                || name_lower.contains("hrm")
                || name_lower.contains("wahoo")
                || name_lower.contains("tickr")
                || name_lower.contains("heart")
                || name_lower.contains("chest")
                || name_lower.contains("huawei")
                || name_lower.contains("watch")
                || name_lower.contains("band")
                || props.services.contains(&HEART_RATE_SERVICE_UUID);
            
            if is_hr_device {
                eprintln!("  -> 识别为心率设备");
                devices.push(BleDevice {
                    id: p.id().to_string(),
                    name,
                    rssi: props.rssi,
                });
            }
        }
    }

    let _ = central.stop_scan().await;
    eprintln!("扫描完成，找到 {} 个心率设备", devices.len());

    Ok(devices)
}

/// 连接到心率设备并监听
pub async fn connect_and_listen_heart_rate(
    device_id: &str,
    callback: impl Fn(HeartRateData) + Send + 'static,
) -> Result<(), String> {
    eprintln!("开始连接设备: {}", device_id);
    
    let manager = Manager::new().await.map_err(|e| format!("创建蓝牙管理器失败: {}", e))?;
    
    let adapters = manager.adapters().await.map_err(|e| format!("获取适配器失败: {}", e))?;
    
    if adapters.is_empty() {
        return Err("未找到蓝牙适配器".to_string());
    }

    let central = adapters.into_iter().next().unwrap();

    // 扫描找设备
    eprintln!("扫描查找设备...");
    central
        .start_scan(ScanFilter::default())
        .await
        .map_err(|e| format!("开始扫描失败: {}", e))?;

    time::sleep(Duration::from_secs(3)).await;

    let peripherals = central.peripherals().await.map_err(|e| format!("获取设备失败: {}", e))?;
    
    let target = peripherals
        .into_iter()
        .find(|p| p.id().to_string() == device_id)
        .ok_or_else(|| "未找到指定设备".to_string())?;

    let _ = central.stop_scan().await;

    // 连接设备
    eprintln!("正在连接...");
    target
        .connect()
        .await
        .map_err(|e| format!("连接设备失败: {}", e))?;
    eprintln!("连接成功!");

    // 发现服务
    eprintln!("发现服务...");
    target
        .discover_services()
        .await
        .map_err(|e| format!("发现服务失败: {}", e))?;

    let characteristics = target.characteristics();
    eprintln!("发现 {} 个特征", characteristics.len());
    
    // 查找心率特征
    let hr_char = characteristics
        .iter()
        .find(|c| {
            eprintln!("检查特征: {} (服务: {})", c.uuid, c.service_uuid);
            c.uuid == HEART_RATE_MEASUREMENT_UUID
        })
        .cloned()
        .ok_or_else(|| {
            eprintln!("未找到心率测量特征，可用特征:");
            for c in &characteristics {
                eprintln!("  - {} (服务: {})", c.uuid, c.service_uuid);
            }
            "未找到心率测量特征".to_string()
        })?;

    eprintln!("找到心率特征: {}", hr_char.uuid);

    // 订阅心率通知
    eprintln!("订阅心率通知...");
    target
        .subscribe(&hr_char)
        .await
        .map_err(|e| format!("订阅心率通知失败: {}", e))?;
    eprintln!("订阅成功! 开始接收数据...");

    // 监听通知
    let mut notifications = target.notifications().await.map_err(|e| e.to_string())?;

    // 处理通知
    tokio::spawn(async move {
        use tokio_stream::StreamExt;
        while let Some(data) = notifications.next().await {
            eprintln!("收到通知数据: {:?}", data.value);
            if let Some(hr_data) = parse_heart_rate_data(&data.value) {
                callback(hr_data);
            }
        }
        eprintln!("通知流结束");
    });

    Ok(())
}
