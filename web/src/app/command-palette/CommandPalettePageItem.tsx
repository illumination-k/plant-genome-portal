import type { ReactElement } from "react";
import { useCallback } from "react";
import CommandPaletteItem from "@/app/command-palette/CommandPaletteItem";

type Page = { detail: string; label: string; to: string };

const CommandPalettePageItem = (props: {
  onSelect: (to: string) => void;
  page: Page;
}): ReactElement => {
  const onSelect = useCallback((): void => {
    props.onSelect(props.page.to);
  }, [props]);

  return (
    <CommandPaletteItem
      detail={props.page.detail}
      label={props.page.label}
      mono={false}
      onSelect={onSelect}
    />
  );
};

export default CommandPalettePageItem;
