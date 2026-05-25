import type { ReactElement } from "react";
import { useCallback, useState } from "react";

const RESET_MS = 1500;

const computeText = (copied: boolean, label: string): string => {
  if (copied) {
    return "Copied";
  }
  return label;
};

const CopyButton = (props: { label?: string; value: string }): ReactElement => {
  const [copied, setCopied] = useState(false);

  const onClick = useCallback(async (): Promise<void> => {
    try {
      await globalThis.navigator.clipboard.writeText(props.value);
      setCopied(true);
      globalThis.setTimeout(() => {
        setCopied(false);
      }, RESET_MS);
    } catch {
      setCopied(false);
    }
  }, [props.value]);

  const label = props.label ?? "Copy";

  return (
    <button
      aria-label={label}
      className="inline-flex items-center gap-1 rounded border border-border-subtle bg-surface px-1.5 py-0.5 font-mono text-[11px] text-text-muted opacity-0 transition group-hover:opacity-100 hover:border-border hover:text-text focus-visible:opacity-100"
      onClick={onClick}
      title={label}
      type="button"
    >
      {computeText(copied, label)}
    </button>
  );
};

export default CopyButton;
