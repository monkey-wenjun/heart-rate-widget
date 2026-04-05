import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import HeartRateChart from './HeartRateChart';

interface BleDevice {
  id: string;
  name: string;
  rssi: number | null;
}

interface HistoryPoint {
  time: number;
  value: number;
}

export default function App() {
  const [isLocked, setIsLocked] = useState(true);
  const [isScanning, setIsScanning] = useState(false);
  const [connectedDevice, setConnectedDevice] = useState<string | null>(null);
  const [deviceName, setDeviceName] = useState<string>('');
  const [heartRate, setHeartRate] = useState<number>(0);
  const [, setSensorContact] = useState(false);
  const [history, setHistory] = useState<HistoryPoint[]>([]);
  const [status, setStatus] = useState<string>('启动中...');

  useEffect(() => {
    const savedLock = localStorage.getItem('hrWidget_locked');
    if (savedLock !== null) {
      setIsLocked(savedLock === 'true');
    }
    
    loadPosition();
    
    (window as any).onHeartRate = (data: any) => {
      console.log('HR:', data);
      setHeartRate(data.heart_rate);
      setSensorContact(data.sensor_contact);
      // 只要有数据就显示已连接
      setStatus('已连接');
      setConnectedDevice(prev => prev || 'connected');
      setHistory(prev => {
        const now = Date.now();
        const newHistory = [...prev, { time: now, value: data.heart_rate }];
        return newHistory.filter(p => now - p.time < 60000);
      });
    };
    
    const handler = (e: any) => {
      const data = e.detail || e.payload;
      if (data) {
        setHeartRate(data.heart_rate);
        setSensorContact(data.sensor_contact);
        setHistory(prev => {
          const now = Date.now();
          const newHistory = [...prev, { time: now, value: data.heart_rate }];
          return newHistory.filter(p => now - p.time < 60000);
        });
      }
    };
    
    window.addEventListener('tauri://event/heart-rate-update', handler);
    autoConnect();
    
    return () => {
      window.removeEventListener('tauri://event/heart-rate-update', handler);
    };
  }, []);

  const loadPosition = async () => {
    try {
      const [x, y] = await invoke<[number, number]>('load_window_position');
      const win = getCurrentWindow();
      await win.setPosition(new (await import('@tauri-apps/api/window')).LogicalPosition(x, y));
    } catch (e) {
      // 默认位置由后端设置
    }
  };

  const autoConnect = async () => {
    setIsScanning(true);
    setStatus('扫描中...');
    
    try {
      const devices = await invoke<BleDevice[]>('scan_devices');
      
      if (devices.length > 0) {
        const device = devices[0];
        setStatus(`连接中...`);
        await connectToDevice(device.id, device.name);
      } else {
        setStatus('未找到设备');
      }
    } catch (err) {
      setStatus('扫描失败');
    } finally {
      setIsScanning(false);
    }
  };

  const connectToDevice = async (deviceId: string, name: string) => {
    try {
      await invoke('connect_device', { deviceId });
      setConnectedDevice(deviceId);
      setDeviceName(name);
      setStatus('连接中...');
      
      localStorage.setItem('hrWidget_lastDevice', deviceId);
      localStorage.setItem('hrWidget_lastDeviceName', name);
    } catch (err) {
      setStatus('连接失败');
    }
  };

  const toggleLock = useCallback(async () => {
    const newLock = !isLocked;
    setIsLocked(newLock);
    localStorage.setItem('hrWidget_locked', String(newLock));
    
    if (!newLock) {
      const win = getCurrentWindow();
      const pos = await win.outerPosition();
      await invoke('save_window_position', { x: pos.x, y: pos.y });
    }
  }, [isLocked]);

  const reconnect = () => {
    setHistory([]);
    setHeartRate(0);
    autoConnect();
  };

  // 锁定图标：锁定时显示 ○，解锁时显示 ●
  const lockIcon = isLocked ? '○' : '●';
  
  return (
    <div style={styles.container}>
      {/* 拖动区域 - 整个背景可拖动（解锁时） */}
      {!isLocked && (
        <div 
          style={styles.dragOverlay}
          data-tauri-drag-region="true"
        />
      )}
      
      {/* 控制按钮 - 右上角浮动 */}
      <div style={styles.controls}>
        <button
          onClick={toggleLock}
          style={styles.iconBtn}
          title={isLocked ? '点击解锁拖动' : '点击锁定位置'}
        >
          {lockIcon}
        </button>
        <button 
          onClick={reconnect}
          style={styles.iconBtn}
          title="重新连接"
        >
          ↻
        </button>
      </div>

      {/* 主内容区 */}
      <div style={styles.content}>
        {/* 状态指示器 */}
        <div style={styles.statusLine}>
          {isScanning && <span style={styles.spinner}>⟳</span>}
          <span style={styles.statusText}>{status}</span>
          {connectedDevice && (
            <span style={styles.dot}>●</span>
          )}
        </div>

        {/* 心率显示 */}
        <div style={styles.hrDisplay}>
          <span style={styles.heart}>❤</span>
          <span style={styles.hrValue}>{heartRate > 0 ? heartRate : '--'}</span>
          <span style={styles.unit}>BPM</span>
        </div>

        {/* 图表 */}
        <div style={styles.chart}>
          <HeartRateChart data={history} />
        </div>
        
        {/* 设备名称 */}
        <div style={styles.deviceName}>
          {deviceName || '等待连接'}
        </div>
      </div>
      
      {/* 解锁提示 */}
      {!isLocked && (
        <div style={styles.dragHint}>
          拖动移动位置
        </div>
      )}
    </div>
  );
}

const styles: { [key: string]: React.CSSProperties } = {
  container: {
    width: '100%',
    height: '100%',
    background: 'rgba(20, 20, 25, 0.05)',
    borderRadius: '12px',
    display: 'flex',
    flexDirection: 'column',
    overflow: 'hidden',
    border: '1px solid rgba(255, 255, 255, 0.08)',
    position: 'relative',
  },
  dragOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    zIndex: 1,
    cursor: 'grab',
  },
  controls: {
    position: 'absolute',
    top: '6px',
    right: '8px',
    display: 'flex',
    gap: '4px',
    zIndex: 10,
  },
  iconBtn: {
    width: '24px',
    height: '24px',
    borderRadius: '4px',
    border: 'none',
    background: 'transparent',
    cursor: 'pointer',
    fontSize: '14px',
    color: 'rgba(255, 255, 255, 0.6)',
    outline: 'none',
    padding: 0,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    transition: 'all 0.2s',
  },
  content: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '24px 16px 12px',
    gap: '8px',
    position: 'relative',
    zIndex: 5,
  },
  statusLine: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    fontSize: '11px',
    height: '16px',
  },
  statusText: {
    color: 'rgba(255, 255, 255, 0.8)',
  },
  spinner: {
    animation: 'spin 1s linear infinite',
    color: 'rgba(255, 255, 255, 0.8)',
  },
  dot: {
    color: '#2ed573',
    fontSize: '8px',
  },
  hrDisplay: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    marginTop: '-4px',
  },
  heart: {
    fontSize: '28px',
    color: '#ff4757',
    filter: 'drop-shadow(0 0 6px rgba(255, 71, 87, 0.6))',
  },
  hrValue: {
    fontSize: '48px',
    fontWeight: '600',
    color: '#ffffff',
    textShadow: '0 0 10px rgba(255, 255, 255, 0.3)',
    fontVariantNumeric: 'tabular-nums',
  },
  unit: {
    fontSize: '14px',
    color: 'rgba(255, 255, 255, 0.6)',
    marginTop: '12px',
  },
  chart: {
    width: '100%',
    height: '50px',
    opacity: 0.8,
  },
  deviceName: {
    fontSize: '10px',
    color: 'rgba(255, 255, 255, 0.5)',
    marginTop: '2px',
  },
  dragHint: {
    position: 'absolute',
    bottom: '4px',
    left: '50%',
    transform: 'translateX(-50%)',
    fontSize: '9px',
    color: 'rgba(255, 255, 255, 0.4)',
    zIndex: 10,
  },
};
