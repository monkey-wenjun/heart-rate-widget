// Prevents additional console window on Windows, DO NOT REMOVE!!
#![windows_subsystem = "windows"]

use std::sync::{Arc, Mutex};
use tauri::{Manager, State};
use tokio::sync::mpsc;

mod ble;
mod system_monitor;
mod logger;
use ble::{connect_and_listen_heart_rate, scan_heart_rate_devices, BleDevice, HeartRateData};
use system_monitor::{SystemMonitor, SystemStats};

struct AppState {
    current_device: Arc<Mutex<Option<String>>>,
    heart_rate_sender: Arc<Mutex<Option<mpsc::Sender<HeartRateData>>>>,
    system_monitor: Arc<SystemMonitor>,
}

// Windows API 设置窗口为桌面底层
#[cfg(target_os = "windows")]
fn set_window_to_desktop_bottom(window: &tauri::WebviewWindow) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_BOTTOM, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    };

    unsafe {
        let hwnd = HWND(window.hwnd().unwrap().0 as *mut std::ffi::c_void);
        // 设置为工具窗口（不显示在任务栏）
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowLongW, SetWindowLongW, GWL_EXSTYLE, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        };
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        SetWindowLongW(
            hwnd,
            GWL_EXSTYLE,
            ex_style | WS_EX_TOOLWINDOW.0 as i32 | WS_EX_NOACTIVATE.0 as i32,
        );

        // 置于底层
        let _ = SetWindowPos(
            hwnd,
            HWND_BOTTOM,
            0,
            0,
            0,
            0,
            SWP_NOSIZE | SWP_NOMOVE | SWP_NOACTIVATE,
        );
    }
}

#[cfg(not(target_os = "windows"))]
fn set_window_to_desktop_bottom(_window: &tauri::WebviewWindow) {
    // 非Windows平台暂无实现
}

// 扫描真实蓝牙心率设备
#[tauri::command]
async fn scan_devices() -> Result<Vec<BleDevice>, String> {
    scan_heart_rate_devices(8).await
}

// 连接设备
#[tauri::command]
async fn connect_device(
    device_id: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    {
        let mut current = state.current_device.lock().unwrap();
        *current = Some(device_id.clone());
    }

    let (tx, mut rx) = mpsc::channel::<HeartRateData>(100);

    {
        let mut sender = state.heart_rate_sender.lock().unwrap();
        *sender = Some(tx.clone());
    }

    // 转发心率数据到前端 - 使用 eval 直接调用全局函数
    let window = app.get_webview_window("main").unwrap();
    tauri::async_runtime::spawn(async move {
        while let Some(hr_data) = rx.recv().await {
            let js = format!(
                "if(window.onHeartRate){{window.onHeartRate({{heart_rate:{},sensor_contact:{}}});}}",
                hr_data.heart_rate,
                if hr_data.sensor_contact { "true" } else { "false" }
            );
            let _ = window.eval(&js);
        }
    });

    let sender_clone = Arc::clone(&state.heart_rate_sender);

    let result = connect_and_listen_heart_rate(&device_id, move |hr_data| {
        if let Ok(sender) = sender_clone.lock() {
            if let Some(tx) = sender.as_ref() {
                let _ = tx.try_send(hr_data);
            }
        }
    })
    .await;

    match result {
        Ok(_) => Ok("已连接".to_string()),
        Err(e) => Err(format!("连接失败: {}", e)),
    }
}

// 断开设备
#[tauri::command]
async fn disconnect_device(state: State<'_, AppState>) -> Result<(), String> {
    {
        let mut current = state.current_device.lock().unwrap();
        *current = None;
    }
    {
        let mut sender = state.heart_rate_sender.lock().unwrap();
        *sender = None;
    }
    Ok(())
}

// 保存窗口位置
#[tauri::command]
async fn save_window_position(x: i32, y: i32) -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .ok_or("无法获取配置目录")?
        .join("heart-rate-widget");

    std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;

    let position_file = config_dir.join("position.json");
    let data = serde_json::json!({"x": x, "y": y});

    std::fs::write(&position_file, data.to_string()).map_err(|e| e.to_string())?;

    Ok(())
}

// 加载窗口位置
#[tauri::command]
async fn load_window_position() -> Result<(i32, i32), String> {
    let config_dir = dirs::config_dir()
        .ok_or("无法获取配置目录")?
        .join("heart-rate-widget");

    let position_file = config_dir.join("position.json");

    if position_file.exists() {
        let content = std::fs::read_to_string(&position_file).map_err(|e| e.to_string())?;

        let data: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        let x = data["x"].as_i64().unwrap_or(100) as i32;
        let y = data["y"].as_i64().unwrap_or(100) as i32;

        Ok((x, y))
    } else {
        Ok((100, 100))
    }
}

// 获取开机自启动状态
#[tauri::command]
async fn get_autostart_status(app: tauri::AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    
    let autostart_manager = app.autolaunch();
    autostart_manager.is_enabled().map_err(|e| e.to_string())
}

// 设置开机自启动
#[tauri::command]
async fn set_autostart(enabled: bool, app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    
    let autostart_manager = app.autolaunch();
    
    if enabled {
        autostart_manager.enable().map_err(|e| e.to_string())?;
    } else {
        autostart_manager.disable().map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

// 获取系统监控数据
#[tauri::command]
async fn get_system_stats(state: State<'_, AppState>) -> Result<SystemStats, String> {
    Ok(state.system_monitor.get_stats())
}

fn main() {
    // 记录启动日志，帮助排查终端弹出问题
    app_log!("=== 应用启动 ===");
    app_log!("版本: 1.2.0");
    app_log!("编译模式: {}", if cfg!(debug_assertions) { "Debug" } else { "Release" });
    
    let system_monitor = Arc::new(SystemMonitor::new());
    
    let state = AppState {
        current_device: Arc::new(Mutex::new(None)),
        heart_rate_sender: Arc::new(Mutex::new(None)),
        system_monitor: system_monitor.clone(),
    };

    tauri::Builder::default()
        // .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
        //     // 当检测到第二个实例时，聚焦到第一个实例的窗口
        // }))
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            scan_devices,
            connect_device,
            disconnect_device,
            save_window_position,
            load_window_position,
            get_autostart_status,
            set_autostart,
            get_system_stats,
        ])
        .setup(move |app| {
            // 启动系统监控
            system_monitor.start_monitoring();
            // 获取主窗口
            let window = app.get_webview_window("main").unwrap();

            // 设置窗口位置（右上角）
            let monitor = window.current_monitor().ok().flatten();
            if let Some(m) = monitor {
                let size = m.size();
                let scale = m.scale_factor();
                let window_width: f64 = 320.0;
                let window_height: f64 = 480.0;
                let right_margin: f64 = 50.0;
                let top_margin: f64 = 50.0;
                
                // 计算逻辑像素位置
                let screen_width = size.width as f64 / scale;
                let screen_height = size.height as f64 / scale;
                
                // 右上角位置，确保不超出屏幕
                let x = (screen_width - window_width - right_margin).max(10.0);
                let y = top_margin.min(screen_height - window_height - 10.0);
                
                let _: Result<(), _> =
                    window.set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
            }

            // 设置窗口为桌面底层（不遮挡其他窗口，不显示在任务栏）
            // 暂时禁用，确保窗口可见
            // set_window_to_desktop_bottom(&window);

            // 创建托盘菜单
            let show_item =
                tauri::menu::MenuItem::with_id(app, "show", "显示/隐藏", true, None::<&str>)?;
            let settings_item =
                tauri::menu::MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
            let quit_item =
                tauri::menu::MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = tauri::menu::Menu::with_items(app, &[&show_item, &settings_item, &quit_item])?;

            // 创建托盘图标（可选，失败不阻塞）
            let _ = (|| -> Result<(), Box<dyn std::error::Error>> {
                let icon = app.default_window_icon().cloned();
                let mut tray_builder = tauri::tray::TrayIconBuilder::new()
                    .menu(&menu)
                    .tooltip("心率监测");
                if let Some(icon) = icon {
                    tray_builder = tray_builder.icon(icon);
                }
                let _tray = tray_builder
                    .on_menu_event(|app: &tauri::AppHandle, event: tauri::menu::MenuEvent| {
                        match event.id().as_ref() {
                            "show" => {
                                if let Some(window) = app.get_webview_window("main") {
                                    let is_visible = window.is_visible().unwrap_or(false);
                                    if is_visible {
                                        let _ = window.hide();
                                    } else {
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }
                                }
                            }
                            "settings" => {
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                    let _ = window.eval("window.showSettings && window.showSettings()");
                                }
                            }
                            "quit" => {
                                app.exit(0);
                            }
                            _ => {}
                        }
                    })
                    .on_tray_icon_event(
                        |tray: &tauri::tray::TrayIcon, event: tauri::tray::TrayIconEvent| {
                            if let tauri::tray::TrayIconEvent::Click {
                                button: tauri::tray::MouseButton::Left,
                                ..
                            } = event
                            {
                                if let Some(window) = tray.app_handle().get_webview_window("main") {
                                    let is_visible = window.is_visible().unwrap_or(false);
                                    if is_visible {
                                        let _ = window.hide();
                                    } else {
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }
                                }
                            }
                        },
                    )
                    .build(app)?;
                Ok(())
            })();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
