# Heart Rate Widget

一个基于 Tauri + React + Rust 开发的蓝牙心率监测桌面小组件，支持 Windows 平台。

![License](https://img.shields.io/badge/license-ISC-blue.svg)
![Tauri](https://img.shields.io/badge/tauri-2.0-blue.svg)
![React](https://img.shields.io/badge/react-18.2-blue.svg)
![Rust](https://img.shields.io/badge/rust-2021-orange.svg)

## 功能特性

- 📊 **实时心率监测** - 通过蓝牙连接心率设备，实时显示 BPM 数据
- 📈 **历史曲线图表** - 显示最近 60 秒的心率变化趋势
- 🔗 **自动连接** - 自动扫描并连接附近的蓝牙心率设备
- 📌 **桌面小组件** - 窗口置顶于桌面，不遮挡其他应用
- 🖱️ **可拖动定位** - 支持解锁拖动，记忆窗口位置
- 🔔 **系统托盘** - 支持最小化到系统托盘，点击托盘图标显示/隐藏
- 🎯 **无任务栏图标** - 工具窗口模式，保持桌面整洁

## 技术栈

- **前端**: React 18 + TypeScript + Vite
- **后端**: Rust + Tauri v2
- **蓝牙**: btleplug 库
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

1. 启动应用后，程序会自动扫描附近的蓝牙心率设备
2. 找到设备后自动连接，开始显示心率数据
3. 点击 🔒 锁定/解锁按钮可以拖动窗口位置
4. 点击 ↻ 按钮可以重新扫描连接设备
5. 右键点击托盘图标可选择显示/隐藏或退出应用

## 项目结构

```
heart-rate-widget/
├── src/                    # 前端源代码
│   ├── App.tsx            # 主应用组件
│   ├── HeartRateChart.tsx # 心率图表组件
│   ├── index.css          # 全局样式
│   └── main.tsx           # 入口文件
├── src-tauri/             # Tauri/Rust 后端
│   ├── src/
│   │   ├── main.rs        # 主程序入口
│   │   └── ble.rs         # 蓝牙模块
│   ├── Cargo.toml         # Rust 依赖配置
│   └── capabilities/      # Tauri 权限配置
├── package.json           # Node.js 依赖配置
├── vite.config.ts         # Vite 配置
└── tsconfig.json          # TypeScript 配置
```

## 支持的设备

任何支持标准蓝牙心率服务（Heart Rate Service, UUID: 0x180D）的设备，例如：

- 小米手环系列
- Apple Watch
- Garmin 心率带
- Polar 心率监测器
- 其他兼容 BLE 心率协议的设备

## 开发说明

### 蓝牙模块

蓝牙功能通过 `btleplug` 库实现，主要功能包括：

- 扫描 BLE 设备并过滤心率服务
- 连接设备并订阅心率测量特征值
- 解析心率数据并实时推送到前端

### 窗口特性

- 使用 Windows API 设置窗口为工具窗口（不显示在任务栏）
- 窗口置于桌面底层，不干扰正常操作
- 支持位置记忆，重启后保持上次位置

## 许可证

[ISC](LICENSE)

## 贡献

欢迎提交 Issue 和 Pull Request！

## 致谢

- [Tauri](https://tauri.app/) - 优秀的跨平台桌面应用框架
- [btleplug](https://github.com/deviceplug/btleplug) - Rust 跨平台蓝牙库
- [React](https://react.dev/) - 前端 UI 库
