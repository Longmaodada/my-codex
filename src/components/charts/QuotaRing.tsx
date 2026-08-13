import { motion } from "framer-motion";
import { useAnimatedNumber } from "../../hooks/useAnimatedNumber";

interface Props {
  value: number | null;
  size?: number;
  stroke?: number;
  label?: string;
  sublabel?: string;
  compact?: boolean;
  healthyThreshold?: number;
  warningThreshold?: number;
}

export function QuotaRing({ value, size = 150, stroke = 12, label = "剩余", sublabel = "7 天额度", compact = false, healthyThreshold = 40, warningThreshold = 20 }: Props) {
  const safeValue = value === null ? 0 : Math.max(0, Math.min(100, value));
  const animated = useAnimatedNumber(safeValue, 980);
  const radius = (size - stroke) / 2;
  const circumference = 2 * Math.PI * radius;
  const dash = circumference * (animated / 100);
  const ringId = `quota-gradient-${size}-${stroke}`;
  const colors = safeValue < warningThreshold
    ? ["#ef4444", "#f87171", "#fb7185"]
    : safeValue <= healthyThreshold
      ? ["#f59e0b", "#f3b23f", "#f97316"]
      : ["#4f5df6", "#7b56f4", "#bd75ed"];

  return (
    <div className={`quota-ring ${compact ? "quota-ring-compact" : ""}`} style={{ width: size, height: size }}>
      <svg viewBox={`0 0 ${size} ${size}`} aria-hidden="true">
        <defs>
          <linearGradient id={ringId} x1="0" y1="1" x2="1" y2="0">
            <stop offset="0%" stopColor={colors[0]} />
            <stop offset="52%" stopColor={colors[1]} />
            <stop offset="100%" stopColor={colors[2]} />
          </linearGradient>
        </defs>
        <circle className="quota-track" cx={size / 2} cy={size / 2} r={radius} strokeWidth={stroke} />
        <motion.circle
          className="quota-progress"
          cx={size / 2}
          cy={size / 2}
          r={radius}
          strokeWidth={stroke}
          stroke={`url(#${ringId})`}
          strokeDasharray={`${dash} ${circumference - dash}`}
          initial={{ strokeDasharray: `0 ${circumference}` }}
          animate={{ strokeDasharray: `${dash} ${circumference - dash}` }}
          transition={{ duration: 0.98, ease: [0.22, 1, 0.36, 1] }}
        />
      </svg>
      <div className="quota-ring-content">
        <strong>{value === null ? "—" : `${Math.round(animated)}%`}</strong>
        <span>{value === null ? "数据不可用" : label}</span>
        {!compact && <small>{sublabel}</small>}
      </div>
    </div>
  );
}
