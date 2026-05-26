/* oxlint-disable no-magic-numbers, id-length, no-ternary, jsx-max-depth, prefer-tag-over-role, max-lines-per-function, max-statements, max-params, max-lines, sort-keys */
import type {
  ClusterDendrogram,
  ClusterDendrogramNode,
  ExpressionClustergramResponse,
} from "@/api/client/types.gen";
import { max } from "d3-array";
import { format } from "d3-format";
import { scaleBand } from "d3-scale";
import { interpolateViridis } from "d3-scale-chromatic";
import type { ReactElement } from "react";
import { useMemo } from "react";

const margin = {
  bottom: 116,
  left: 188,
  right: 28,
  top: 116,
};
const rowLabelWidth = 96;
const dendrogramGap = 12;
const rowDendrogramWidth = 80;
const columnDendrogramHeight = 76;
const cellSize = 30;
const minCellSize = 18;
const maxCellSize = 34;
const labelMaxLength = 18;
const valueFormat = format(".2~f");

type Point = {
  x: number;
  y: number;
};

type DendrogramLink = {
  first: Point;
  second: Point;
  parent: Point;
};

const unitLabel = (unit: string): string => unit.replace("_", " ").toUpperCase();

const shorten = (value: string): string =>
  value.length > labelMaxLength ? `${value.slice(0, labelMaxLength - 1)}...` : value;

const matrixOffset = (
  matrix: ExpressionClustergramResponse,
  geneIndex: number,
  sampleIndex: number,
): number => geneIndex * matrix.samples.length + sampleIndex;

const zColor = (value: number, zMax: number): string => {
  if (zMax === 0) {
    return interpolateViridis(0.5);
  }
  const clamped = Math.max(-1, Math.min(1, value / zMax));
  return interpolateViridis((clamped + 1) / 2);
};

const nodeChildren = (node: ClusterDendrogramNode): [number, number] | undefined => {
  if (
    node.left === undefined ||
    node.left === null ||
    node.right === undefined ||
    node.right === null
  ) {
    return undefined;
  }
  return [node.left, node.right];
};

const dendrogramMaxDistance = (dendrogram: ClusterDendrogram): number =>
  Math.max(max(dendrogram.nodes, (node) => node.distance) ?? 0, 1);

const internalNodes = (dendrogram: ClusterDendrogram): ClusterDendrogramNode[] =>
  dendrogram.nodes.filter((node) => nodeChildren(node) !== undefined);

const rowNodePoint = (
  dendrogram: ClusterDendrogram,
  nodeId: number,
  leafY: Map<number, number>,
  cache: Map<number, Point>,
  heatmapLeft: number,
): Point => {
  const cached = cache.get(nodeId);
  if (cached) {
    return cached;
  }
  const node = dendrogram.nodes[nodeId];
  if (!node) {
    return { x: heatmapLeft, y: margin.top };
  }
  if (node.leafIndex !== undefined && node.leafIndex !== null) {
    const point = { x: heatmapLeft, y: leafY.get(node.leafIndex) ?? margin.top };
    cache.set(nodeId, point);
    return point;
  }
  const children = nodeChildren(node);
  if (!children) {
    return { x: heatmapLeft, y: margin.top };
  }
  const first = rowNodePoint(dendrogram, children[0], leafY, cache, heatmapLeft);
  const second = rowNodePoint(dendrogram, children[1], leafY, cache, heatmapLeft);
  const distance = node.distance / dendrogramMaxDistance(dendrogram);
  const point = {
    x: heatmapLeft - dendrogramGap - distance * rowDendrogramWidth,
    y: (first.y + second.y) / 2,
  };
  cache.set(nodeId, point);
  return point;
};

const columnNodePoint = (
  dendrogram: ClusterDendrogram,
  nodeId: number,
  leafX: Map<number, number>,
  cache: Map<number, Point>,
  heatmapTop: number,
): Point => {
  const cached = cache.get(nodeId);
  if (cached) {
    return cached;
  }
  const node = dendrogram.nodes[nodeId];
  if (!node) {
    return { x: margin.left, y: heatmapTop };
  }
  if (node.leafIndex !== undefined && node.leafIndex !== null) {
    const point = { x: leafX.get(node.leafIndex) ?? margin.left, y: heatmapTop };
    cache.set(nodeId, point);
    return point;
  }
  const children = nodeChildren(node);
  if (!children) {
    return { x: margin.left, y: heatmapTop };
  }
  const first = columnNodePoint(dendrogram, children[0], leafX, cache, heatmapTop);
  const second = columnNodePoint(dendrogram, children[1], leafX, cache, heatmapTop);
  const distance = node.distance / dendrogramMaxDistance(dendrogram);
  const point = {
    x: (first.x + second.x) / 2,
    y: heatmapTop - dendrogramGap - distance * columnDendrogramHeight,
  };
  cache.set(nodeId, point);
  return point;
};

const rowLinks = (
  dendrogram: ClusterDendrogram,
  leafY: Map<number, number>,
  heatmapLeft: number,
): DendrogramLink[] => {
  const cache = new Map<number, Point>();
  return internalNodes(dendrogram).map((node) => {
    const children = nodeChildren(node) ?? [node.id, node.id];
    return {
      first: rowNodePoint(dendrogram, children[0], leafY, cache, heatmapLeft),
      parent: rowNodePoint(dendrogram, node.id, leafY, cache, heatmapLeft),
      second: rowNodePoint(dendrogram, children[1], leafY, cache, heatmapLeft),
    };
  });
};

const columnLinks = (
  dendrogram: ClusterDendrogram,
  leafX: Map<number, number>,
  heatmapTop: number,
): DendrogramLink[] => {
  const cache = new Map<number, Point>();
  return internalNodes(dendrogram).map((node) => {
    const children = nodeChildren(node) ?? [node.id, node.id];
    return {
      first: columnNodePoint(dendrogram, children[0], leafX, cache, heatmapTop),
      parent: columnNodePoint(dendrogram, node.id, leafX, cache, heatmapTop),
      second: columnNodePoint(dendrogram, children[1], leafX, cache, heatmapTop),
    };
  });
};

const ExpressionClustergram = (props: { matrix: ExpressionClustergramResponse }): ReactElement => {
  const chart = useMemo(() => {
    const orderedGenes = props.matrix.rowOrder.map((index) => props.matrix.genes[index]);
    const orderedSamples = props.matrix.columnOrder.map((index) => props.matrix.samples[index]);
    const size = Math.max(
      minCellSize,
      Math.min(maxCellSize, cellSize - Math.max(0, orderedSamples.length - 12)),
    );
    const chartWidth = margin.left + orderedSamples.length * size + margin.right;
    const chartHeight = margin.top + orderedGenes.length * size + margin.bottom;
    const x = scaleBand(
      orderedSamples.map((sample) => sample.run),
      [margin.left, chartWidth - margin.right],
    ).paddingInner(0.04);
    const y = scaleBand(
      orderedGenes.map((gene) => gene.geneId),
      [margin.top, chartHeight - margin.bottom],
    ).paddingInner(0.04);
    const zMax = Math.max(max(props.matrix.zScores.map((value) => Math.abs(value))) ?? 0, 1);
    const leafY = new Map<number, number>();
    const leafX = new Map<number, number>();
    for (const geneIndex of props.matrix.rowOrder) {
      const gene = props.matrix.genes[geneIndex];
      if (gene) {
        leafY.set(geneIndex, (y(gene.geneId) ?? margin.top) + y.bandwidth() / 2);
      }
    }
    for (const sampleIndex of props.matrix.columnOrder) {
      const sample = props.matrix.samples[sampleIndex];
      if (sample) {
        leafX.set(sampleIndex, (x(sample.run) ?? margin.left) + x.bandwidth() / 2);
      }
    }

    return {
      columnLinks: columnLinks(props.matrix.columnDendrogram, leafX, margin.top),
      chartHeight,
      chartWidth,
      orderedGenes,
      orderedSamples,
      rowLinks: rowLinks(props.matrix.rowDendrogram, leafY, margin.left),
      x,
      y,
      zMax,
    };
  }, [props.matrix]);

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
