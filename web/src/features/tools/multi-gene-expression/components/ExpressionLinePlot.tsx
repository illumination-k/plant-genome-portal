/* oxlint-disable no-magic-numbers, id-length, no-ternary, jsx-max-depth, prefer-tag-over-role, max-lines-per-function */
import type { ExpressionClustergramResponse } from "@/api/client/types.gen";
import { max } from "d3-array";
import { format } from "d3-format";
import { scaleLinear, scalePoint } from "d3-scale";
import type { ReactElement } from "react";
import { useMemo } from "react";

const chartHeight = 360;
const margin = {
  bottom: 88,
  left: 64,
  right: 108,
  top: 28,
};
const sampleGap = 72;
const minChartWidth = 760;
const axisTickCount = 5;
const minDomainMax = 1;
const pointRadius = 3.5;
const labelMaxLength = 16;
const palette = ["#2f7d57", "#2f6fbb", "#c77800", "#8a4a8e", "#b64a5f", "#5f6f2f"];
const valueFormat = format(".2~f");

const unitLabel = (unit: string): string => unit.replace("_", " ").toUpperCase();

const shorten = (value: string): string =>
  value.length > labelMaxLength ? `${value.slice(0, labelMaxLength - 1)}...` : value;

const matrixValue = (
  matrix: ExpressionClustergramResponse,
  geneIndex: number,
  sampleIndex: number,
): number => matrix.values[geneIndex * matrix.samples.length + sampleIndex] ?? 0;

const ExpressionLinePlot = (props: { matrix: ExpressionClustergramResponse }): ReactElement => {
  const chart = useMemo(() => {
    const orderedSamples = props.matrix.columnOrder.map((index) => props.matrix.samples[index]);
    const chartWidth = Math.max(
      minChartWidth,
      orderedSamples.length * sampleGap + margin.left + margin.right,
    );
    const x = scalePoint(
      orderedSamples.map((sample) => sample.run),
      [margin.left, chartWidth - margin.right],
    ).padding(0.35);
    const yMax = Math.max(max(props.matrix.values) ?? minDomainMax, minDomainMax);
    const y = scaleLinear([0, yMax], [chartHeight - margin.bottom, margin.top]).nice();

    return {
      chartWidth,
      orderedSamples,
      ticks: y.ticks(axisTickCount),
      x,
      y,
    };
  }, [props.matrix]);

  const plotBottom = chartHeight - margin.bottom;
  const plotRight = chart.chartWidth - margin.right;

  return (
    <div className="w-full overflow-x-auto">
      <svg
        aria-label={`Expression line plot in ${unitLabel(props.matrix.unit)}`}
        className="h-auto"
        role="img"
        viewBox={`0 0 ${chart.chartWidth} ${chartHeight}`}
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
          {unitLabel(props.matrix.unit)}
        </text>
        {chart.orderedSamples.map((sample) => {
          const x = chart.x(sample.run) ?? margin.left;
          return (
            <g key={sample.run}>
              <line
                stroke="var(--border-subtle)"
                strokeWidth="1"
                x1={x}
                x2={x}
                y1={plotBottom}
                y2={plotBottom + 5}
              />
              <text
                fill="var(--text-muted)"
                fontSize="11"
                textAnchor="end"
                transform={`translate(${x - 4} ${plotBottom + 18}) rotate(-38)`}
              >
                {shorten(sample.label)}
              </text>
            </g>
          );
        })}
        {props.matrix.genes.map((gene, geneIndex) => {
          const color = palette[geneIndex % palette.length] ?? palette[0];
          const points = props.matrix.columnOrder.map((sampleIndex) => {
            const sample = props.matrix.samples[sampleIndex];
            return {
              sample,
              value: matrixValue(props.matrix, geneIndex, sampleIndex),
              x: chart.x(sample.run) ?? margin.left,
            };
          });
          const path = points
            .map((point, index) => `${index === 0 ? "M" : "L"}${point.x},${chart.y(point.value)}`)
            .join(" ");

          return (
            <g key={gene.geneId}>
              <path d={path} fill="none" stroke={color} strokeWidth="2.5" />
              {points.map((point) => (
                <circle
                  cx={point.x}
                  cy={chart.y(point.value)}
                  fill={color}
                  key={`${gene.geneId}-${point.sample.run}`}
                  r={pointRadius}
                  stroke="var(--surface)"
                  strokeWidth="1.5"
                >
                  <title>{`${gene.label} / ${point.sample.label}: ${valueFormat(point.value)} ${unitLabel(props.matrix.unit)}`}</title>
                </circle>
              ))}
              <g transform={`translate(${plotRight + 16} ${margin.top + geneIndex * 20})`}>
                <line stroke={color} strokeWidth="2.5" x1="0" x2="18" y1="5" y2="5" />
                <text fill="var(--text-muted)" fontSize="12" x="24" y="9">
                  {shorten(gene.label)}
                </text>
              </g>
            </g>
          );
        })}
      </svg>
    </div>
  );
};

export default ExpressionLinePlot;
