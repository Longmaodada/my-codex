import { useId } from "react";
import {
  Area,
  AreaChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { DailyUsage } from "../../types/analytics";
import { formatTokens } from "../../utils/format";

export function UsageTrend({ data }: { data: DailyUsage[] }) {
  const id = useId().replaceAll(":", "");
  const chartData = data.map((day) => ({ ...day, label: day.date.slice(5) }));
  return (
    <div className="usage-chart" aria-label="Token 使用趋势">
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={chartData} margin={{ top: 8, right: 8, left: -18, bottom: 0 }}>
          <defs>
            <linearGradient id={`area-${id}`} x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="#6e5ff5" stopOpacity={0.36} />
              <stop offset="100%" stopColor="#6e5ff5" stopOpacity={0.015} />
            </linearGradient>
          </defs>
          <CartesianGrid stroke="rgba(96, 82, 156, .09)" vertical={false} />
          <XAxis dataKey="label" tickLine={false} axisLine={false} minTickGap={30} tick={{ fontSize: 10, fill: "var(--text-muted)" }} />
          <YAxis tickFormatter={(value) => formatTokens(value, 0)} tickLine={false} axisLine={false} width={42} tick={{ fontSize: 10, fill: "var(--text-muted)" }} />
          <Tooltip
            cursor={{ stroke: "rgba(91, 75, 234, .25)", strokeDasharray: "4 4" }}
            formatter={(value, name) => [formatTokens(Number(value)), String(name)]}
            labelFormatter={(_, payload) => payload?.[0]?.payload?.date ?? ""}
            contentStyle={{ border: "1px solid var(--border)", borderRadius: 14, background: "var(--tooltip)", backdropFilter: "blur(20px)", fontSize: 11 }}
          />
          <Area type="monotone" dataKey="input" name="输入" stroke="#6572ee" strokeWidth={1.1} fillOpacity={0} animationDuration={700} />
          <Area type="monotone" dataKey="output" name="输出" stroke="#f09ad8" strokeWidth={1.1} fillOpacity={0} animationDuration={760} />
          <Area type="monotone" dataKey="reasoning" name="思考" stroke="#edb554" strokeWidth={1.1} fillOpacity={0} animationDuration={820} />
          <Area type="monotone" dataKey="total" name="总计" stroke="#6557e8" strokeWidth={2.4} fill={`url(#area-${id})`} animationDuration={850} />
          <Area type="monotone" dataKey="cached" name="缓存" stroke="#a271ee" strokeWidth={1.25} fillOpacity={0} animationDuration={980} />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}
