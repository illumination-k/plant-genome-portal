import type { ReactElement } from "react";

const variantClass = (mono: boolean): string => {
  if (mono) {
    return "font-mono text-sm text-text";
  }
  return "text-sm text-text";
};

const CommandPaletteItem = (props: {
  detail: string;
  label: string;
  mono: boolean;
  onSelect: () => void;
}): ReactElement => (
  <button
    className="flex w-full items-center justify-between gap-3 rounded-md px-2 py-1.5 text-left hover:bg-surface-muted focus-visible:bg-surface-muted focus-visible:outline-none"
    onClick={props.onSelect}
    type="button"
  >
    <span className={variantClass(props.mono)}>{props.label}</span>
    <span className="truncate text-[12px] text-text-subtle">{props.detail}</span>
  </button>
);

export default CommandPaletteItem;
