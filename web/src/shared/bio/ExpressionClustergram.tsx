/* oxlint-disable no-magic-numbers, id-length, jsx-max-depth, prefer-tag-over-role, max-lines-per-function, no-ternary */
import type { ExpressionClustergramResponse } from "@/api/client/types.gen";
import { format } from "d3-format";
import { interpolateViridis } from "d3-scale-chromatic";
import type { ReactElement } from "react";
import { useMemo } from "react";
import {
  buildClustergramLayout,
  labelMaxLength,
  margin,
  matrixOffset,
  rowLabelWidth,
} from "./expressionClustergramLayout";

const valueFormat = format(".2~f");

const unitLabel = (unit: string): string => unit.replace("_", " ").toUpperCase();

const shorten = (value: string): string =>
  value.length > labelMaxLength ? `${value.slice(0, labelMaxLength - 1)}...` : value;

const zColor = (value: number, zMax: number): string => {
  if (zMax === 0) {
    return interpolateViridis(0.5);
  }
  const clamped = Math.max(-1, Math.min(1, value / zMax));
  return interpolateViridis((clamped + 1) / 2);
};

const ExpressionClustergram = (props: { matrix: ExpressionClustergramResponse }): ReactElement => {
  const chart = useMemo(() => buildClustergramLayout(props.matrix), [props.matrix]);

  return (
    <div className="w-full overflow-x-auto">
      <svg
        aria-label={`Expression clustergram in ${unitLabel(props.matrix.unit)}`}
        className="mx-auto block"
        height={chart.chartHeight}
        role="img"
        viewBox={`0 0 ${chart.chartWidth} ${chart.chartHeight}`}
        width={chart.chartWidth}
      >
        <g transform={`translate(${margin.left} 24)`}>
          <text fill="var(--text-muted)" fontSize="11" x="0" y="-8">
            Row z-score
          </text>
          {[-1, 0, 1].map((value, index) => (
            <rect
              fill={zColor(value * chart.zMax, chart.zMax)}
              height="10"
              key={value}
              width="34"
              x={index * 34}
              y="0"
            />
          ))}
        </g>
        <g fill="none" stroke="var(--border-strong)" strokeLinecap="round" strokeWidth="1.25">
          {chart.columnLinks.map((link) => (
            <path
              d={`M${link.first.x},${link.first.y} V${link.parent.y} H${link.second.x} V${link.second.y}`}
              key={`column-${link.parent.x}-${link.parent.y}`}
            />
          ))}
          {chart.rowLinks.map((link) => (
            <path
              d={`M${link.first.x},${link.first.y} H${link.parent.x} V${link.second.y} H${link.second.x}`}
              key={`row-${link.parent.x}-${link.parent.y}`}
            />
          ))}
        </g>
        {chart.orderedSamples.map((sample) => {
          const x = chart.x(sample.run) ?? margin.left;
          return (
            <text
              fill="var(--text-muted)"
              fontSize="11"
              key={sample.run}
              textAnchor="end"
              transform={`translate(${x + chart.x.bandwidth() / 2 - 4} ${chart.chartHeight - margin.bottom + 18}) rotate(-42)`}
            >
              {shorten(sample.label)}
            </text>
          );
        })}
        {chart.orderedGenes.map((gene) => {
          const y = chart.y(gene.geneId) ?? margin.top;
          return (
            <text
              dominantBaseline="middle"
              fill="var(--text-muted)"
              fontSize="12"
              key={gene.geneId}
              textAnchor="end"
              x={rowLabelWidth - 10}
              y={y + chart.y.bandwidth() / 2}
            >
              {shorten(gene.label)}
            </text>
          );
        })}
        {props.matrix.rowOrder.flatMap((geneIndex) =>
          props.matrix.columnOrder.map((sampleIndex) => {
            const gene = props.matrix.genes[geneIndex];
            const sample = props.matrix.samples[sampleIndex];
            const offset = matrixOffset(props.matrix, geneIndex, sampleIndex);
            const value = props.matrix.values[offset] ?? 0;
            const zScore = props.matrix.zScores[offset] ?? 0;
            const x = chart.x(sample.run) ?? margin.left;
            const y = chart.y(gene.geneId) ?? margin.top;
            return (
              <rect
                fill={zColor(zScore, chart.zMax)}
                height={chart.y.bandwidth()}
                key={`${gene.geneId}-${sample.run}`}
                width={chart.x.bandwidth()}
                x={x}
                y={y}
              >
                <title>{`${gene.label} / ${sample.label}: ${valueFormat(value)} ${unitLabel(props.matrix.unit)}; z=${valueFormat(zScore)}`}</title>
              </rect>
            );
          }),
        )}
      </svg>
    </div>
  );
};

export default ExpressionClustergram;
