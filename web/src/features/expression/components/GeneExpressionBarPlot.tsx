/* oxlint-disable no-magic-numbers, id-length, no-ternary, max-lines-per-function, jsx-max-depth, prefer-tag-over-role */
import type { GeneExpressionPoint } from "@/api/client/types.gen";
import { max } from "d3-array";
import { format } from "d3-format";
import { scaleBand, scaleLinear } from "d3-scale";
import type { ReactElement } from "react";
import { useMemo } from "react";

const chartWidth = 760;
const chartHeight = 360;
const margin = {
  bottom: 76,
  left: 64,
  right: 24,
  top: 28,
};
const axisTickCount = 5;
const minDomainMax = 1;
const labelMaxLength = 18;
const palette = ["#2f7d57", "#2f6fbb", "#9a5f00", "#8a4a8e", "#b64a5f"];
const valueFormat = format(".2~f");
const maxPointOffset = 18;
const pointRadius = 4;
const errorCapWidth = 18;

type ExpressionGroup = {
  label: string;
  mean: number;
  points: GeneExpressionPoint[];
  sd: number;
};

const unitLabel = (unit: string): string => unit.replace("_", " ").toUpperCase();

const shorten = (value: string): string =>
  value.length > labelMaxLength ? `${value.slice(0, labelMaxLength - 1)}...` : value;

const groupLabel = (point: GeneExpressionPoint): string => point.primaryGroup ?? point.label;

const colorFor = (group: string, groups: string[]): string => {
  const index = groups.indexOf(group);
  return palette[index % palette.length] ?? palette[0];
};

const mean = (values: number[]): number =>
  values.reduce((sum, value) => sum + value, 0) / Math.max(values.length, 1);

const sampleSd = (values: number[], avg: number): number => {
  if (values.length < 2) {
    return 0;
  }

  const variance = values.reduce((sum, value) => sum + (value - avg) ** 2, 0) / (values.length - 1);
  return Math.sqrt(variance);
};

const buildGroups = (points: GeneExpressionPoint[]): ExpressionGroup[] => {
  const byLabel = new Map<string, GeneExpressionPoint[]>();
  for (const point of points) {
    const label = groupLabel(point);
    byLabel.set(label, [...(byLabel.get(label) ?? []), point]);
  }

  return Array.from(byLabel, ([label, groupPoints]) => {
    const values = groupPoints.map((point) => point.value);
    const avg = mean(values);
    return {
      label,
      mean: avg,
      points: groupPoints,
      sd: sampleSd(values, avg),
    };
  });
};

const GeneExpressionBarPlot = (props: { points: GeneExpressionPoint[] }): ReactElement => {
  const chart = useMemo(() => {
    const data = buildGroups(props.points);
    const labels = data.map((group) => group.label);
    const maxValue =
      max(data, (group) =>
        Math.max(group.mean + group.sd, ...group.points.map((point) => point.value)),
      ) ?? minDomainMax;
    const yMax = Math.max(maxValue, minDomainMax);
    const x = scaleBand(labels, [margin.left, chartWidth - margin.right]).padding(0.34);
    const y = scaleLinear([0, yMax], [chartHeight - margin.bottom, margin.top]).nice();

    return {
      data,
      labels,
      ticks: y.ticks(axisTickCount),
      x,
      y,
    };
  }, [props.points]);

  const unit = props.points[0]?.unit ?? "tpm";
  const plotBottom = chartHeight - margin.bottom;
  const plotRight = chartWidth - margin.right;

  return (
    <div className="w-full overflow-x-auto">
      <svg
        aria-label={`Gene expression bar plot in ${unitLabel(unit)}`}
        className="h-auto min-w-[620px]"
        role="img"
        viewBox={`0 0 ${chartWidth} ${chartHeight}`}
      >
        <line
          stroke="var(--border)"
          strokeWidth="1"
          x1={margin.left}
          x2={plotRight}
          y1={plotBottom}
          y2={plotBottom}
        />
        <line
          stroke="var(--border)"
          strokeWidth="1"
          x1={margin.left}
          x2={margin.left}
          y1={margin.top}
          y2={plotBottom}
        />
        {chart.ticks.map((tick) => {
          const y = chart.y(tick);
          return (
            <g key={tick}>
              <line
                stroke="var(--border-subtle)"
                strokeWidth="1"
                x1={margin.left}
                x2={plotRight}
                y1={y}
                y2={y}
              />
              <text
                dominantBaseline="middle"
                fill="var(--text-muted)"
                fontSize="12"
                textAnchor="end"
                x={margin.left - 10}
                y={y}
              >
                {valueFormat(tick)}
              </text>
            </g>
          );
        })}
        <text
          fill="var(--text-muted)"
          fontSize="12"
          textAnchor="middle"
          transform={`translate(18 ${margin.top + (plotBottom - margin.top) / 2}) rotate(-90)`}
        >
          {unitLabel(unit)}
        </text>
        {chart.data.map((group) => {
          const x = chart.x(group.label) ?? margin.left;
          const width = chart.x.bandwidth();
          const centerX = x + width / 2;
          const y = chart.y(group.mean);
          const height = plotBottom - y;
          const errorHighY = chart.y(group.mean + group.sd);
          const errorLowY = chart.y(Math.max(0, group.mean - group.sd));
          const color = colorFor(group.label, chart.labels);
          return (
            <g key={group.label}>
              <rect fill={color} height={height} opacity="0.28" rx="4" width={width} x={x} y={y}>
                <title>{`${group.label}: mean ${valueFormat(group.mean)} ${unitLabel(unit)}, SD ${valueFormat(group.sd)}`}</title>
              </rect>
              {group.sd > 0 && (
                <g stroke={color} strokeLinecap="round" strokeWidth="2">
                  <line x1={centerX} x2={centerX} y1={errorHighY} y2={errorLowY} />
                  <line
                    x1={centerX - errorCapWidth / 2}
                    x2={centerX + errorCapWidth / 2}
                    y1={errorHighY}
                    y2={errorHighY}
                  />
                  <line
                    x1={centerX - errorCapWidth / 2}
                    x2={centerX + errorCapWidth / 2}
                    y1={errorLowY}
                    y2={errorLowY}
                  />
                </g>
              )}
              {group.points.map((point, index) => {
                const step = Math.min(maxPointOffset, width / (group.points.length + 1));
                const offset = (index - (group.points.length - 1) / 2) * step;
                return (
                  <circle
                    cx={centerX + offset}
                    cy={chart.y(point.value)}
                    fill={color}
                    key={point.run}
                    r={pointRadius}
                    stroke="var(--surface)"
                    strokeWidth="1.5"
                  >
                    <title>{`${point.label}: ${valueFormat(point.value)} ${unitLabel(point.unit)}`}</title>
                  </circle>
                );
              })}
              <text
                fill="var(--text-muted)"
                fontSize="11"
                textAnchor="middle"
                x={centerX}
                y={plotBottom + 20}
              >
                {shorten(group.label)}
              </text>
            </g>
          );
        })}
      </svg>
    </div>
  );
};

export default GeneExpressionBarPlot;
