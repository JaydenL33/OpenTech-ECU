import type { ReactNode } from "react";

export function Gauge({
  label,
  value,
  min,
  max,
  unit,
  color = "var(--accent)",
  decimals = 0,
  warning,
}: {
  label: string;
  value: number | null;
  min: number;
  max: number;
  unit: string;
  color?: string;
  decimals?: number;
  warning?: boolean;
}) {
  const v = value ?? 0;
  const t = Math.max(0, Math.min(1, (v - min) / (max - min || 1)));
  const start = -220;
  const end = 40;
  const angle = start + (end - start) * t;
  const cx = 90;
  const cy = 88;
  const r = 64;
  const polar = (deg: number, radius: number) => {
    const rad = (deg * Math.PI) / 180;
    return [cx + Math.cos(rad) * radius, cy + Math.sin(rad) * radius];
  };
  const arc = (from: number, to: number, radius: number) => {
    const [x1, y1] = polar(from, radius);
    const [x2, y2] = polar(to, radius);
    const large = Math.abs(to - from) > 180 ? 1 : 0;
    return `M ${x1} ${y1} A ${radius} ${radius} 0 ${large} 1 ${x2} ${y2}`;
  };
  const [nx, ny] = polar(angle, r - 10);
  return (
    <div className={`gauge ${warning ? "gauge-warning" : ""}`}>
      <svg viewBox="0 0 180 112" role="img" aria-label={`${label} ${value ?? "unavailable"}`}>
        <path d={arc(start, end, r)} fill="none" stroke="#e8e8e8" strokeWidth="9" strokeLinecap="round" />
        <path d={arc(start, angle, r)} fill="none" stroke={warning ? "var(--danger)" : color} strokeWidth="9" strokeLinecap="round" />
        <line x1={cx} y1={cy} x2={nx} y2={ny} stroke={warning ? "var(--danger)" : color} strokeWidth="2" />
        <circle cx={cx} cy={cy} r="3.5" fill={warning ? "var(--danger)" : color} />
      </svg>
      <div className="gauge-value">
        {value === null || !Number.isFinite(value) ? "—" : value.toFixed(decimals)}
        <span>{unit}</span>
      </div>
      <div className="gauge-label">{label}</div>
    </div>
  );
}

export function Stat({
  label,
  value,
  unit,
  tone = "",
}: {
  label: string;
  value: ReactNode;
  unit?: string;
  tone?: string;
}) {
  return (
    <div className={`stat ${tone}`}>
      <span className="stat-label">{label}</span>
      <span className="stat-value">
        {value}
        {unit ? <small>{unit}</small> : null}
      </span>
    </div>
  );
}

export function Badge({
  children,
  tone = "neutral",
}: {
  children: ReactNode;
  tone?: "neutral" | "good" | "warn" | "danger" | "info";
}) {
  return <span className={`badge badge-${tone}`}>{children}</span>;
}

export function Panel({
  title,
  subtitle,
  actions,
  children,
  className = "",
}: {
  title?: string;
  subtitle?: string;
  actions?: ReactNode;
  children: ReactNode;
  className?: string;
}) {
  return (
    <section className={`panel ${className}`}>
      {(title || actions) && (
        <header className="panel-header">
          <div>
            {title && <h2>{title}</h2>}
            {subtitle && <p>{subtitle}</p>}
          </div>
          {actions && <div className="panel-actions">{actions}</div>}
        </header>
      )}
      {children}
    </section>
  );
}

export function SummaryCard({
  icon,
  title,
  subtitle,
  action,
  children,
  className = "",
}: {
  icon?: ReactNode;
  title: string;
  subtitle?: string;
  action?: ReactNode;
  children: ReactNode;
  className?: string;
}) {
  return (
    <section className={`summary-card ${className}`}>
      <header className="summary-head">
        {icon && <span className="summary-icon">{icon}</span>}
        <div>
          <h2>{title}</h2>
          {subtitle && <p>{subtitle}</p>}
        </div>
      </header>
      <div className="summary-body">{children}</div>
      {action && <div className="summary-more">{action}</div>}
    </section>
  );
}

export function SummaryRow({
  label,
  value,
  tone = "",
}: {
  label: string;
  value: ReactNode;
  tone?: "" | "warn" | "danger" | "good";
}) {
  return (
    <div className={`summary-row ${tone}`}>
      <span>{label}</span>
      <b>{value}</b>
    </div>
  );
}

export function MoreLink({ onClick, children }: { onClick: () => void; children: ReactNode }) {
  return (
    <button className="more-link" onClick={onClick}>
      {children} <span aria-hidden="true">→</span>
    </button>
  );
}
