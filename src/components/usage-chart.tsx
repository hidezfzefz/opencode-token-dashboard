"use client";

import {
  Area,
  AreaChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

interface ChartPoint {
  day: string;
  fresh: number;
  cache: number;
}

function compact(value: number): string {
  return new Intl.NumberFormat("en-US", { notation: "compact", maximumFractionDigits: 1 }).format(value);
}

export function UsageChart({ data }: { data: ChartPoint[] }) {
  return (
    <div style={{ width: "100%", height: 280 }} role="img" aria-label="Stacked daily token usage, fresh tokens and cache reads">
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={data} margin={{ top: 8, right: 8, bottom: 0, left: 0 }}>
          <CartesianGrid stroke="var(--cv-line)" vertical={false} />
          <XAxis
            dataKey="day"
            tick={{ fill: "var(--cv-text-50)", fontSize: 12 }}
            tickLine={false}
            axisLine={{ stroke: "var(--cv-line)" }}
            minTickGap={24}
          />
          <YAxis
            tickFormatter={compact}
            tick={{ fill: "var(--cv-text-50)", fontSize: 12 }}
            tickLine={false}
            axisLine={false}
            width={52}
          />
          <Tooltip
            contentStyle={{
              background: "var(--cv-card-2)",
              border: "1px solid var(--cv-line)",
              borderRadius: 12,
              color: "var(--cv-text)",
              fontSize: 14,
            }}
            labelStyle={{ color: "var(--cv-text)" }}
            formatter={(value, name) => [compact(Number(value)), name === "fresh" ? "Fresh" : "Cache read"]}
          />
          <Area
            type="monotone"
            dataKey="fresh"
            stackId="usage"
            stroke="var(--cv-text-75)"
            fill="var(--cv-text-50)"
            fillOpacity={0.5}
            strokeWidth={1.5}
          />
          <Area
            type="monotone"
            dataKey="cache"
            stackId="usage"
            stroke="var(--cv-text-25)"
            fill="var(--cv-raised)"
            fillOpacity={0.9}
            strokeWidth={1}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}
