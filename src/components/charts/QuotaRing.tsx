import { motion } from "framer-motion";
import { useAnimatedNumber } from "../../hooks/useAnimatedNumber";

interface Props {
  value: number | null;
  size?: number;
  stroke?: number;
  label?: string;
  sublabel?: string;
  compact?: boolean;
}

export function quotaColor(value: number | null) {
  if (value === null) return "#8c8799";
  if (value > 75) return "#3267ff";
  if (value > 50) return "#28b979";
  if (value > 25) return "#f39a28";
  return "#ef4b45";
}

export function QuotaArc({ value, size = 150, stroke = 12 }: { value: number | null; size?: number; stroke?: number }) {
  const animated = useAnimatedNumber(value === null ? 0 : Math.max(0, Math.min(100, value)), 980);
  const radius = (size - stroke) / 2;
  const circumference = 2 * Math.PI * radius;
  const color = quotaColor(value);
  const dash = circumference * (animated / 100);

  return (
    <svg viewBox={`0 0 ${size} ${size}`} aria-hidden="true">
      <circle className="quota-track" cx={size / 2} cy={size / 2} r={radius} strokeWidth={stroke} />
      <motion.circle className="quota-progress" cx={size / 2} cy={size / 2} r={radius} stroke={color} strokeWidth={stroke} strokeDasharray={`${dash} ${circumference}`} initial={{ strokeDasharray: `0 ${circumference}` }} animate={{ strokeDasharray: `${dash} ${circumference}` }} transition={{ duration: 0.98, ease: [0.22, 1, 0.36, 1] }} />
    </svg>
  );
}

export function QuotaRing({ value, size = 150, stroke = 12, label = "剩余", sublabel = "7 天额度", compact = false }: Props) {
  const safeValue = value === null ? 0 : Math.max(0, Math.min(100, value));

  return (
    <div className={`quota-ring ${compact ? "quota-ring-compact" : ""}`} style={{ width: size, height: size }}>
      <QuotaArc value={value} size={size} stroke={stroke} />
      <div className="quota-ring-content">
        <strong>{value === null ? "—" : `${Math.round(safeValue)}%`}</strong>
        <span>{value === null ? "数据不可用" : label}</span>
        {!compact && <small>{sublabel}</small>}
      </div>
    </div>
  );
}
