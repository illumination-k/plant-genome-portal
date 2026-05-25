import type { Cds, Exon, GeneRecord, Strand, Transcript } from "@/api/client/types.gen";

const VIEWBOX_WIDTH = 1000;
const HORIZONTAL_PADDING = 16;
const ROW_HEIGHT = 56;
const ROW_LABEL_Y = 12;
const ROW_TRACK_CENTER_Y = 38;
const AXIS_HEIGHT = 28;
const MIN_BOX_WIDTH = 1.2;
const EXON_HALF_HEIGHT = 5;
const CDS_HALF_HEIGHT = 9;
const CHEVRON_HALF = 4;
const CHEVRON_MIN_INTRON_PIXELS = 28;
const CHEVRON_SPACING_PIXELS = 60;
const TICK_HEIGHT = 4;
const TICK_LABEL_OFFSET = 12;
const STROKE_INTRON = 1.5;
const STROKE_CHEVRON = 1.2;
const STROKE_EXON = 0.8;
const STROKE_TICK = 1;
const RECT_RADIUS = 1;
const MIN_SPAN = 1;
const PADDING_SIDES = 2;
const EMPTY_LENGTH = 0;
const ZERO_INDEX = 0;
const ONE = 1;
const HALF_DIVISOR = 2;
const REVERSE_CHEVRON_SIGN = -1;
const FORWARD_CHEVRON_SIGN = 1;
const TICKS_PER_AXIS = 3;

type Scale = (pos: number) => number;

type BoxRect = {
  height: number;
  key: string;
  posX: number;
  posY: number;
  title: string;
  width: number;
};

type ChevronToken = { key: string; pathD: string };

type IntronLineToken = { posY: number; x1: number; x2: number };

type LabelToken = { id: string; posX: number; posY: number; title: string };

type TrackTokens = {
  cdsBoxes: BoxRect[];
  chevrons: ChevronToken[];
  exonBoxes: BoxRect[];
  intronLine: IntronLineToken;
  label: LabelToken;
};

type TickAnchor = "start" | "middle" | "end";

type AxisTick = { anchor: TickAnchor; label: string; posX: number };

type AxisTokens = {
  lineX1: number;
  lineX2: number;
  posY: number;
  ticks: AxisTick[];
};

type TranscriptGroup = {
  cdss: Cds[];
  exons: Exon[];
  transcript: Transcript;
};

type IntronSegment = { end: number; start: number };

type BoxKind = "exon" | "cds";

type BoxConfig = {
  centerY: number;
  extra: string;
  halfHeight: number;
  kind: BoxKind;
  region: { end: number; start: number };
  scale: Scale;
};

type AxisConfig = {
  end: number;
  posY: number;
  scale: Scale;
  start: number;
};

const makeScale = (start: number, end: number): Scale => {
  const span = Math.max(end - start, MIN_SPAN);
  const usable = VIEWBOX_WIDTH - HORIZONTAL_PADDING * PADDING_SIDES;
  return (pos: number) => HORIZONTAL_PADDING + ((pos - start) / span) * usable;
};

const boxWidth = (scale: Scale, start: number, end: number): number =>
  Math.max(scale(end) - scale(start), MIN_BOX_WIDTH);

const compareByStart = (
  left: { region: { start: number } },
  right: { region: { start: number } },
): number => left.region.start - right.region.start;

const groupByTranscript = (record: GeneRecord): TranscriptGroup[] =>
  record.transcripts.map((transcript) => ({
    cdss: record.cdss.filter((cds) => cds.transcript_id === transcript.id).toSorted(compareByStart),
    exons: record.exons
      .filter((exon) => exon.transcript_id === transcript.id)
      .toSorted(compareByStart),
    transcript,
  }));

const walkExonGaps = (
  startPosition: number,
  exons: Exon[],
): { cursor: number; segments: IntronSegment[] } => {
  const segments: IntronSegment[] = [];
  let cursor = startPosition;
  for (const exon of exons) {
    if (exon.region.start > cursor) {
      segments.push({ end: exon.region.start, start: cursor });
    }
    cursor = Math.max(cursor, exon.region.end);
  }
  return { cursor, segments };
};

const intronSegments = (transcript: Transcript, exons: Exon[]): IntronSegment[] => {
  if (exons.length === EMPTY_LENGTH) {
    return [{ end: transcript.region.end, start: transcript.region.start }];
  }
  const walked = walkExonGaps(transcript.region.start, exons);
  if (walked.cursor < transcript.region.end) {
    walked.segments.push({ end: transcript.region.end, start: walked.cursor });
  }
  return walked.segments;
};

const strandChevronSign = (strand: Strand): number => {
  if (strand === "reverse") {
    return REVERSE_CHEVRON_SIGN;
  }
  return FORWARD_CHEVRON_SIGN;
};

const chevronPath = (centerX: number, centerY: number, strand: Strand): string => {
  const offset = strandChevronSign(strand) * CHEVRON_HALF;
  const topY = centerY - CHEVRON_HALF;
  const bottomY = centerY + CHEVRON_HALF;
  return `M ${centerX - offset} ${topY} L ${centerX} ${centerY} L ${centerX - offset} ${bottomY}`;
};

const chevronPositions = (scale: Scale, start: number, end: number): number[] => {
  const startPx = scale(start);
  const endPx = scale(end);
  const widthPx = endPx - startPx;
  if (widthPx < CHEVRON_MIN_INTRON_PIXELS) {
    return [];
  }
  const count = Math.max(ONE, Math.floor(widthPx / CHEVRON_SPACING_PIXELS));
  const step = widthPx / (count + ONE);
  return Array.from({ length: count }, (_unused, index) => startPx + step * (index + ONE));
};

const phaseSuffix = (phase: number | null | undefined): string => {
  if (typeof phase === "number") {
    return `, phase ${phase}`;
  }
  return "";
};

const labelForKind = (kind: BoxKind): string => {
  if (kind === "cds") {
    return "CDS";
  }
  return "Exon";
};

const boxFromRegion = (config: BoxConfig): BoxRect => {
  const span = config.region.end - config.region.start;
  return {
    height: config.halfHeight * PADDING_SIDES,
    key: `${config.kind}-${config.region.start}-${config.region.end}`,
    posX: config.scale(config.region.start),
    posY: config.centerY - config.halfHeight,
    title: `${labelForKind(config.kind)} ${config.region.start + ONE}-${config.region.end} (${span} bp${config.extra})`,
    width: boxWidth(config.scale, config.region.start, config.region.end),
  };
};

const trackTokens = (group: TranscriptGroup, scale: Scale, rowIndex: number): TrackTokens => {
  const trackY = rowIndex * ROW_HEIGHT + ROW_TRACK_CENTER_Y;
  const { transcript } = group;
  const introns = intronSegments(transcript, group.exons);
  const chevrons: ChevronToken[] = introns.flatMap((intron) =>
    chevronPositions(scale, intron.start, intron.end).map((centerX) => ({
      key: `${transcript.id}-chev-${intron.start}-${centerX}`,
      pathD: chevronPath(centerX, trackY, transcript.strand),
    })),
  );
  return {
    cdsBoxes: group.cdss.map((cds) =>
      boxFromRegion({
        centerY: trackY,
        extra: phaseSuffix(cds.phase),
        halfHeight: CDS_HALF_HEIGHT,
        kind: "cds",
        region: cds.region,
        scale,
      }),
    ),
    chevrons,
    exonBoxes: group.exons.map((exon) =>
      boxFromRegion({
        centerY: trackY,
        extra: "",
        halfHeight: EXON_HALF_HEIGHT,
        kind: "exon",
        region: exon.region,
        scale,
      }),
    ),
    intronLine: {
      posY: trackY,
      x1: scale(transcript.region.start),
      x2: scale(transcript.region.end),
    },
    label: {
      id: transcript.id,
      posX: scale(transcript.region.start),
      posY: rowIndex * ROW_HEIGHT + ROW_LABEL_Y,
      title: `${transcript.id} · ${transcript.feature_type}`,
    },
  };
};

const tickAnchor = (index: number, count: number): TickAnchor => {
  if (index === ZERO_INDEX) {
    return "start";
  }
  if (index === count - ONE) {
    return "end";
  }
  return "middle";
};

const formatTickLabel = (pos: number): string => new Intl.NumberFormat("en-US").format(pos + ONE);

const axisTokens = (config: AxisConfig): AxisTokens => {
  const span = config.end - config.start;
  const midpoint = config.start + Math.floor(span / HALF_DIVISOR);
  const positions = [config.start, midpoint, config.end - ONE];
  return {
    lineX1: HORIZONTAL_PADDING,
    lineX2: VIEWBOX_WIDTH - HORIZONTAL_PADDING,
    posY: config.posY,
    ticks: positions.map((pos, index) => ({
      anchor: tickAnchor(index, TICKS_PER_AXIS),
      label: formatTickLabel(pos),
      posX: config.scale(pos),
    })),
  };
};

const totalSvgHeight = (transcriptCount: number): number =>
  Math.max(transcriptCount, MIN_SPAN) * ROW_HEIGHT + AXIS_HEIGHT;

const computeAxisY = (transcriptCount: number): number =>
  transcriptCount * ROW_HEIGHT + AXIS_HEIGHT / TICKS_PER_AXIS;

const isEmpty = (count: number): boolean => count === EMPTY_LENGTH;

const geneStructure = {
  RECT_RADIUS,
  STROKE_CHEVRON,
  STROKE_EXON,
  STROKE_INTRON,
  STROKE_TICK,
  TICK_HEIGHT,
  TICK_LABEL_OFFSET,
  VIEWBOX_WIDTH,
  axisTokens,
  computeAxisY,
  groupByTranscript,
  isEmpty,
  makeScale,
  totalSvgHeight,
  trackTokens,
};

export default geneStructure;
