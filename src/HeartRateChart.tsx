import { useMemo } from 'react';

interface DataPoint {
  time: number;
  value: number;
}

interface Props {
  data: DataPoint[];
}

export default function HeartRateChart({ data }: Props) {
  const color = '#ff4757'; // 固定红色
  
  const svgContent = useMemo(() => {
    if (data.length < 2) {
      return (
        <text
          x="50%"
          y="50%"
          textAnchor="middle"
          dominantBaseline="middle"
          fill="#444"
          fontSize="12"
        >
          等待心率数据...
        </text>
      );
    }

    const width = 348;
    const height = 80;
    const padding = { top: 10, right: 10, bottom: 10, left: 10 };
    const chartWidth = width - padding.left - padding.right;
    const chartHeight = height - padding.top - padding.bottom;

    // 获取时间范围
    const now = Date.now();
    const timeWindow = 60000; // 60秒
    const minTime = now - timeWindow;

    // 过滤并获取数值范围
    const visibleData = data.filter(d => d.time >= minTime);
    if (visibleData.length < 2) {
      return (
        <text
          x="50%"
          y="50%"
          textAnchor="middle"
          dominantBaseline="middle"
          fill="#444"
          fontSize="12"
        >
          收集数据中...
        </text>
      );
    }

    const values = visibleData.map(d => d.value);
    const minValue = Math.max(40, Math.min(...values) - 10);
    const maxValue = Math.min(200, Math.max(...values) + 10);
    const valueRange = maxValue - minValue || 1;

    // 生成路径
    const points = visibleData.map(d => {
      const x = padding.left + ((d.time - minTime) / timeWindow) * chartWidth;
      const y = padding.top + chartHeight - ((d.value - minValue) / valueRange) * chartHeight;
      return `${x},${y}`;
    });

    const pathD = `M ${points.join(' L ')}`;

    // 生成填充区域路径
    const areaD = `${pathD} L ${padding.left + chartWidth},${padding.top + chartHeight} L ${padding.left},${padding.top + chartHeight} Z`;

    // 生成网格线
    const gridLines = [];
    for (let i = 0; i <= 4; i++) {
      const y = padding.top + (chartHeight / 4) * i;
      gridLines.push(
        <line
          key={`grid-${i}`}
          x1={padding.left}
          y1={y}
          x2={padding.left + chartWidth}
          y2={y}
          stroke="rgba(255,255,255,0.05)"
          strokeWidth={1}
          strokeDasharray="2,4"
        />
      );
    }

    return (
      <>
        {gridLines}
        {/* 填充区域 */}
        <path
          d={areaD}
          fill={`${color}15`}
        />
        {/* 线条 */}
        <path
          d={pathD}
          fill="none"
          stroke={color}
          strokeWidth={2}
          strokeLinecap="round"
          strokeLinejoin="round"
        />
        {/* 数据点已移除 */}
        {/* 当前值标签 */}
        {visibleData.length > 0 && (
          <>
            <text
              x={padding.left + chartWidth - 5}
              y={padding.top + 15}
              textAnchor="end"
              fill={color}
              fontSize="10"
              fontWeight="600"
            >
              {maxValue}
            </text>
            <text
              x={padding.left + chartWidth - 5}
              y={padding.top + chartHeight - 5}
              textAnchor="end"
              fill="#666"
              fontSize="10"
            >
              {minValue}
            </text>
          </>
        )}
      </>
    );
  }, [data]);

  return (
    <svg
      width="100%"
      height="100%"
      viewBox="0 0 348 80"
      preserveAspectRatio="none"
      style={{ overflow: 'visible' }}
    >
      {svgContent}
    </svg>
  );
}
