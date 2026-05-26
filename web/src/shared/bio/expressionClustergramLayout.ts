/* oxlint-disable no-magic-numbers, id-length, max-statements, max-params, import/exports-last, import/no-named-export, import/group-exports, typescript/explicit-module-boundary-types, typescript/explicit-function-return-type */
import type {
  ClusterDendrogram,
  ClusterDendrogramNode,
  ExpressionClustergramResponse,
} from "@/api/client/types.gen";
import { max } from "d3-array";
import { scaleBand } from "d3-scale";

export const margin = {
  bottom: 116,
  left: 188,
  right: 28,
  top: 116,
};
export const rowLabelWidth = 96;
export const labelMaxLength = 18;

const dendrogramGap = 12;
const rowDendrogramWidth = 80;
const columnDendrogramHeight = 76;
const cellSize = 30;
const minCellSize = 18;
const maxCellSize = 34;

type Point = {
  x: number;
  y: number;
};

type DendrogramLink = {
  first: Point;
  second: Point;
  parent: Point;
};

export const matrixOffset = (
  matrix: ExpressionClustergramResponse,
  geneIndex: number,
  sampleIndex: number,
): number => geneIndex * matrix.samples.length + sampleIndex;

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

export const buildClustergramLayout = (matrix: ExpressionClustergramResponse) => {
  const orderedGenes = matrix.rowOrder.map((index) => matrix.genes[index]);
  const orderedSamples = matrix.columnOrder.map((index) => matrix.samples[index]);
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
  const zMax = Math.max(max(matrix.zScores.map((value) => Math.abs(value))) ?? 0, 1);
  const leafY = new Map<number, number>();
  const leafX = new Map<number, number>();

  for (const geneIndex of matrix.rowOrder) {
    const gene = matrix.genes[geneIndex];
    if (gene) {
      leafY.set(geneIndex, (y(gene.geneId) ?? margin.top) + y.bandwidth() / 2);
    }
  }
  for (const sampleIndex of matrix.columnOrder) {
    const sample = matrix.samples[sampleIndex];
    if (sample) {
      leafX.set(sampleIndex, (x(sample.run) ?? margin.left) + x.bandwidth() / 2);
    }
  }

  return {
    chartHeight,
    chartWidth,
    columnLinks: columnLinks(matrix.columnDendrogram, leafX, margin.top),
    orderedGenes,
    orderedSamples,
    rowLinks: rowLinks(matrix.rowDendrogram, leafY, margin.left),
    x,
    y,
    zMax,
  };
};
