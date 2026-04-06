import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface SystemStats {
  cpu_percent: number;
  memory_percent: number;
  memory_used_gb: number;
  memory_total_gb: number;
  gpu_percent: number | null;
  gpu_memory_used_mb: number | null;
  gpu_memory_total_mb: number | null;
}

interface HistoryPoint {
  time: number;
  value: number;
}

interface SystemMonitorProps {
  showCpu: boolean;
  showMemory: boolean;
  showGpu: boolean;
}

export default function SystemMonitor({ showCpu, showMemory, showGpu }: SystemMonitorProps) {
  const [stats, setStats] = useState<SystemStats>({
    cpu_percent: 0,
    memory_percent: 0,
    memory_used_gb: 0,
    memory_total_gb: 0,
    gpu_percent: null,
    gpu_memory_used_mb: null,
    gpu_memory_total_mb: null,
  });
  
  const [cpuHistory, setCpuHistory] = useState<HistoryPoint[]>([]);
  const [memHistory, setMemHistory] = useState<HistoryPoint[]>([]);
  const intervalRef = useRef<number | null>(null);

  useEffect(() => {
    if (!showCpu && !showMemory && !showGpu) {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
      return;
    }

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

  if (!showCpu && !showMemory && !showGpu) return null;

  return (
    <div style={styles.container}>
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
      {showGpu && (
        <MetricCard
          title="显存"
          value={stats.gpu_percent ?? 0}
          unit="%"
          color="#ffe66d"
          history={[]}
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
}

function MetricCard({ title, value, unit, subValue, color, history }: MetricCardProps) {
  return (
    <div style={styles.card}>
      <div style={styles.cardHeader}>
        <span style={styles.cardTitle}>{title}</span>
        <div style={styles.valueGroup}>
          <span style={{ ...styles.cardValue, color }}>
            {value.toFixed(0)}{unit}
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
  if (data.length < 2) return null;
  
  // 使用百分比宽度，自适应容器
  const maxVal = Math.max(...data.map(d => d.value), 100);
  
  const points = data.map((d, i) => {
    const x = (i / (data.length - 1)) * 100;
    const y = 100 - (d.value / maxVal) * 100;
    return `${x},${y}`;
  }).join(' ');

  return (
    <svg width="100%" height="35" viewBox="0 0 100 100" preserveAspectRatio="none">
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
    padding: '8px 12px 12px',
    boxSizing: 'border-box',
  },
  card: {
    background: 'rgba(255, 255, 255, 0.08)',
    borderRadius: '8px',
    padding: '10px 12px',
    display: 'flex',
    flexDirection: 'column',
    gap: '6px',
    width: '100%',
    boxSizing: 'border-box',
    border: '1px solid rgba(255, 255, 255, 0.1)',
  },
  cardHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    width: '100%',
  },
  cardTitle: {
    fontSize: '12px',
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
    fontSize: '16px',
    fontWeight: '700',
  },
  subValue: {
    fontSize: '10px',
    color: 'rgba(255, 255, 255, 0.5)',
  },
  chart: {
    height: '35px',
    width: '100%',
    display: 'flex',
    alignItems: 'flex-end',
    justifyContent: 'center',
    overflow: 'hidden',
  },
};
