import { useApp } from "../context";
import { evaluateWarnings, useWarningLimits } from "../warnings";
import { Badge } from "./ui";

export function WarningBanner() {
  const app = useApp();
  const limits = useWarningLimits();
  const warnings = evaluateWarnings(app.live.frame, limits);
  if (warnings.length === 0) return null;

  return (
    <div className="alert danger warning-banner" role="alert">
      <strong>CHANNEL LIMIT WARNING</strong>
      <div className="warning-list">
        {warnings.map((warning) => (
          <span key={`${warning.key}-${warning.limit}`}>
            <Badge tone="danger">{warning.label}</Badge>
            {warning.value.toFixed(warning.decimals)} {warning.unit} is {warning.limit === "high" ? "at or above" : "at or below"}{" "}
            the {warning.limit} limit {warning.threshold.toFixed(warning.decimals)} {warning.unit}
          </span>
        ))}
      </div>
      <span className="warning-footnote">
        Browser-only visual warning. No calibration write, physical output or ECU protection action is performed.
      </span>
    </div>
  );
}
