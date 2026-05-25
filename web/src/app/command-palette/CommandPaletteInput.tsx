import type { ChangeEvent, ReactElement } from "react";
import { useCallback } from "react";

const CommandPaletteInput = (props: {
  onChange: (value: string) => void;
  value: string;
}): ReactElement => {
  const onInput = useCallback(
    (event: ChangeEvent<HTMLInputElement>): void => {
      props.onChange(event.target.value);
    },
    [props],
  );

  return (
    <input
      aria-label="Search genes and pages"
      autoComplete="off"
      className="h-12 w-full border-b border-border-subtle bg-transparent px-4 font-mono text-sm text-text outline-none placeholder:text-text-subtle"
      onChange={onInput}
      placeholder="Search genes, pages…"
      spellCheck={false}
      type="text"
      value={props.value}
    />
  );
};

export default CommandPaletteInput;
