import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface SystemStats {
  cpu_percent: number;
  memory_percent: number;
  memory_used_gb: number;
  memory_total_gb: number;
  gpu_percent: number;
  gpu_memory_used_mb: number;
  gpu_memory_total_mb: number;
  has_gpu: boolean;
}

interface HistoryPoint {
  time: number;
  value: number;
}

interface MetricsPanelProps {
  heartRate: number;
  showHeartRate: boolean;
  showCpu: boolean;
  showMemory: boolean;
  showGpu: boolean;
}

export default function MetricsPanel({ 
  heartRate, 
  showHeartRate, 
  showCpu, 
  showMemory, 
  showGpu 
}: MetricsPanelProps) {
  const [stats, setStats] = useState<SystemStats>({
    cpu_percent: 0,
    memory_percent: 0,
    memory_used_gb: 0,
    memory_total_gb: 0,
    gpu_percent: 0,
    gpu_memory_used_mb: 0,
    gpu_memory_total_mb: 0,
    has_gpu: false,
  });
  
  const [cpuHistory, setCpuHistory] = useState<HistoryPoint[]>([]);
  const [memHistory, setMemHistory] = useState<HistoryPoint[]>([]);
  const [gpuHistory, setGpuHistory] = useState<HistoryPoint[]>([]);
  const [hrHistory, setHrHistory] = useState<HistoryPoint[]>([]);
  const intervalRef = useRef<number | null>(null);

  useEffect(() => {
    const fetchStats = async () => {
      try {
        const data = await invoke<SystemStats>('get_system_stats');
        setStats(data);
        
        const now = Date.now();
        
        if (showCpu) {
          setCpuHistory(prev => {
            const newHistory = [...prev, { time: now, value: data.cpu_percent }];
            return newHistory.filter(p => now - p.time < 60000);
          });
        }
        
        if (showMemory) {
          setMemHistory(prev => {
            const newHistory = [...prev, { time: now, value: data.memory_percent }];
            return newHistory.filter(p => now - p.time < 60000);
          });
        }
        
        if (showGpu && data.has_gpu) {
          setGpuHistory(prev => {
            const newHistory = [...prev, { time: now, value: data.gpu_percent }];
            return newHistory.filter(p => now - p.time < 60000);
          });
        }
      } catch (e) {
        console.error('获取系统状态失败:', e);
      }
    };

    fetchStats();
    intervalRef.current = window.setInterval(fetchStats, 2000);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [showCpu, showMemory, showGpu]);
  
  // 心率历史
  useEffect(() => {
    if (heartRate > 0 && showHeartRate) {
      const now = Date.now();
      setHrHistory(prev => {
        const newHistory = [...prev, { time: now, value: heartRate }];
        return newHistory.filter(p => now - p.time < 60000);
      });
    }
  }, [heartRate, showHeartRate]);

  const hasAnyMetric = showHeartRate || showCpu || showMemory || showGpu;
  if (!hasAnyMetric) return null;

  return (
    <div style={styles.container}>
      {showHeartRate && (
        <MetricCard
          title="心率"
          value={heartRate > 0 ? heartRate : 0}
          unit="BPM"
          subValue={heartRate > 0 ? undefined : '等待连接'}
          color="#ff4757"
          history={hrHistory}
          icon="❤"
        />
      )}
      {showCpu && (
        <MetricCard
          title="CPU"
          value={stats.cpu_percent}
          unit="%"
          color="#ff6b6b"
          history={cpuHistory}
        />
      )}
      {showMemory && (
        <MetricCard
          title="内存"
          value={stats.memory_percent}
          unit="%"
          subValue={`${stats.memory_used_gb.toFixed(1)}/${stats.memory_total_gb.toFixed(1)} GB`}
          color="#4ecdc4"
          history={memHistory}
        />
      )}
      {showGpu && stats.has_gpu && (
        <MetricCard
          title="显存"
          value={stats.gpu_percent}
          unit="%"
          subValue={`${(stats.gpu_memory_used_mb / 1024.0).toFixed(1)}/${(stats.gpu_memory_total_mb / 1024.0).toFixed(1)} GB`}
          color="#ffe66d"
          history={gpuHistory}
        />
      )}
    </div>
  );
}

interface MetricCardProps {
  title: string;
  value: number;
  unit: string;
  subValue?: string;
  color: string;
  history: HistoryPoint[];
  icon?: string;
}

function MetricCard({ title, value, unit, subValue, color, history, icon }: MetricCardProps) {
  const displayValue = value > 0 ? value.toFixed(0) : '--';
  
  return (
    <div style={styles.card}>
      <div style={styles.cardHeader}>
        <div style={styles.titleGroup}>
          {icon && <span style={{ ...styles.icon, color }}>{icon}</span>}
          <span style={styles.cardTitle}>{title}</span>
        </div>
        <div style={styles.valueGroup}>
          <span style={{ ...styles.cardValue, color }}>
            {displayValue}{unit}
          </span>
          {subValue && (
            <span style={styles.subValue}>{subValue}</span>
          )}
        </div>
      </div>
      <div style={styles.chart}>
        <MiniChart data={history} color={color} />
      </div>
    </div>
  );
}

function MiniChart({ data, color }: { data: HistoryPoint[]; color: string }) {
  if (data.length < 2) {
    return <div style={styles.emptyChart}>等待数据...</div>;
  }
  
  const maxVal = Math.max(...data.map(d => d.value), 100);
  const minVal = Math.min(...data.map(d => d.value), 0);
  const range = maxVal - minVal || 1;
  
  const points = data.map((d, i) => {
    const x = (i / (data.length - 1)) * 100;
    const y = 100 - ((d.value - minVal) / range) * 100;
    return `${x},${y}`;
  }).join(' ');

  return (
    <svg width="100%" height="40" viewBox="0 0 100 100" preserveAspectRatio="none">
      <defs>
        <linearGradient id={`grad-${color.replace('#', '')}`} x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stopColor={color} stopOpacity="0.3" />
          <stop offset="100%" stopColor={color} stopOpacity="0" />
        </linearGradient>
      </defs>
      <polygon
        points={`0,100 ${points} 100,100`}
        fill={`url(#grad-${color.replace('#', '')})`}
      />
      <polyline
        points={points}
        fill="none"
        stroke={color}
        strokeWidth="3"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

const styles: { [key: string]: React.CSSProperties } = {
  container: {
    width: '100%',
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
    padding: '8px 12px',
    boxSizing: 'border-box',
    overflowY: 'auto',
    overflowX: 'hidden',
    flex: 1,
  },
  card: {
    background: 'rgba(255, 255, 255, 0.08)',
    borderRadius: '10px',
    padding: '12px 14px',
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
    width: '100%',
    boxSizing: 'border-box',
    border: '1px solid rgba(255, 255, 255, 0.1)',
    boxShadow: '0 2px 8px rgba(0, 0, 0, 0.2)',
  },
  cardHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    width: '100%',
  },
  titleGroup: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  icon: {
    fontSize: '16px',
    filter: 'drop-shadow(0 0 4px currentColor)',
  },
  cardTitle: {
    fontSize: '13px',
    color: 'rgba(255, 255, 255, 0.8)',
    fontWeight: '600',
  },
  valueGroup: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'flex-end',
    gap: '2px',
  },
  cardValue: {
    fontSize: '18px',
    fontWeight: '700',
  },
  subValue: {
    fontSize: '10px',
    color: 'rgba(255, 255, 255, 0.5)',
  },
  chart: {
    height: '40px',
    width: '100%',
    display: 'flex',
    alignItems: 'flex-end',
    justifyContent: 'center',
    overflow: 'hidden',
  },
  emptyChart: {
    height: '40px',
    width: '100%',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '10px',
    color: 'rgba(255, 255, 255, 0.3)',
  },
};
