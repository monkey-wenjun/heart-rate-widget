# Heart Rate Widget

一个基于 Tauri + React + Rust 开发的蓝牙心率监测桌面小组件，支持 Windows 平台。

![Heart Rate Widget Screenshot](https://file.awen.me/images/clip_1775384844303_s5chuh.png)

![License](https://img.shields.io/badge/license-ISC-blue.svg)
![Tauri](https://img.shields.io/badge/tauri-2.0-blue.svg)
![React](https://img.shields.io/badge/react-18.2-blue.svg)
![Rust](https://img.shields.io/badge/rust-2021-orange.svg)

## 功能特性

- 📊 **实时心率监测** - 通过蓝牙连接心率设备，实时显示 BPM 数据
- 📈 **历史曲线图表** - 显示最近 60 秒的心率变化趋势
- 🔗 **蓝牙设备扫描** - 自动扫描并连接附近的蓝牙心率设备
- 💻 **系统监控** - 实时显示 CPU、内存、GPU 使用率
- 📌 **桌面小组件** - 透明背景，无边框窗口，保持桌面整洁
- 🖱️ **可拖动定位** - 支持解锁拖动，记忆窗口位置
- 🔔 **系统托盘** - 支持最小化到系统托盘，点击托盘图标显示/隐藏
- 🚀 **开机自启动** - 支持设置开机自动启动
- 🎯 **无任务栏图标** - 工具窗口模式，不占用任务栏空间

## 技术栈

- **前端**: React 18 + TypeScript + Vite
- **后端**: Rust + Tauri v2
- **蓝牙**: btleplug 库
- **系统监控**: sysinfo + Windows API
- **UI**: 自定义 CSS 样式

## 环境要求

- Windows 10/11
- Node.js 18+
- Rust 1.70+
- 支持 BLE（低功耗蓝牙）的硬件设备

## 安装与运行

### 1. 克隆仓库

```bash
git clone git@github.com:monkey-wenjun/heart-rate-widget.git
cd heart-rate-widget
```

### 2. 安装依赖

```bash
npm install
```

### 3. 开发模式运行

```bash
npm run tauri:dev
```

### 4. 构建生产版本

```bash
npm run tauri:build
```

构建完成后，安装包位于 `src-tauri/target/release/bundle/` 目录下。

## 使用方法

1. 启动应用后，点击扫描按钮查找附近的蓝牙心率设备
2. 选择设备并连接，开始显示实时心率数据
3. 点击锁定/解锁按钮可以拖动窗口位置
4. 窗口位置会自动保存，下次启动时恢复
5. 在设置中可以开启/关闭开机自启动
6. 点击托盘图标可以显示/隐藏应用窗口

## 项目结构

```
heart-rate-widget/
├── src/                    # 前端源代码 (React + TypeScript)
│   ├── App.tsx            # 主应用组件
│   ├── HeartRateChart.tsx # 心率图表组件
│   ├── index.css          # 全局样式
│   └── main.tsx           # 入口文件
├── src-tauri/             # Tauri/Rust 后端
│   ├── src/
│   │   ├── main.rs        # 主程序入口
│   │   ├── ble.rs         # 蓝牙模块
│   │   └── system_monitor.rs  # 系统监控模块
│   ├── Cargo.toml         # Rust 依赖配置
│   ├── tauri.conf.json    # Tauri 配置
│   └── icons/             # 应用图标
├── package.json           # Node.js 依赖配置
├── vite.config.ts         # Vite 配置
└── tsconfig.json          # TypeScript 配置
```

## 核心模块说明

### 蓝牙模块 (`src-tauri/src/ble.rs`)

蓝牙功能通过 `btleplug` 库实现，主要功能包括：

- 扫描 BLE 设备并过滤心率服务 (UUID: 0x180D)
- 连接设备并订阅心率测量特征值 (UUID: 0x2A37)
- 解析心率数据协议，支持：
  - 8位/16位心率值格式
  - 传感器接触状态检测
  - 能量消耗数据
  - RR 间隔数据
- 实时推送心率数据到前端

### 系统监控模块 (`src-tauri/src/system_monitor.rs`)

实时监控系统资源使用情况：

- **CPU 使用率** - 全局 CPU 占用百分比
- **内存使用** - 已用/总内存 (GB) 及百分比
- **GPU 监控** - 支持 NVIDIA (nvidia-smi)、AMD (rocm-smi) 及通用显卡

### 窗口特性 (`src-tauri/src/main.rs`)

- 使用 Windows API 设置窗口为工具窗口（不显示在任务栏）
- 透明背景，无边框设计
- 默认定位于屏幕右上角
- 支持开机自启动配置

## 支持的设备

任何支持标准蓝牙心率服务（Heart Rate Service, UUID: 0x180D）的设备，例如：

- Polar 心率监测器 (H10, H9, OH1 等)
- Garmin 心率带
- Wahoo TICKR 系列
- 小米手环系列
- Huawei Watch / Band
- Apple Watch
- 其他兼容 BLE 心率协议的设备

## 开发说明

### 前端开发

前端使用 React + TypeScript 开发，主要组件：

- `App.tsx` - 主应用逻辑，处理蓝牙连接和数据显示
- `HeartRateChart.tsx` - 心率曲线图表组件

### 后端开发

Rust 后端提供以下 Tauri 命令：

- `scan_devices()` - 扫描蓝牙心率设备
- `connect_device(device_id)` - 连接指定设备
- `disconnect_device()` - 断开当前连接
- `save_window_position(x, y)` - 保存窗口位置
- `load_window_position()` - 加载窗口位置
- `get_autostart_status()` - 获取自启动状态
- `set_autostart(enabled)` - 设置自启动
- `get_system_stats()` - 获取系统监控数据

## 许可证

[ISC](LICENSE)

## 贡献

欢迎提交 Issue 和 Pull Request！

## 致谢

- [Tauri](https://tauri.app/) - 优秀的跨平台桌面应用框架
- [btleplug](https://github.com/deviceplug/btleplug) - Rust 跨平台蓝牙库
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) - 系统信息库
- [React](https://react.dev/) - 前端 UI 库
