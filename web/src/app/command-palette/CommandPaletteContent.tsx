import type { ReactElement } from "react";
import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router";
import CommandPaletteBody from "@/app/command-palette/CommandPaletteBody";
import CommandPaletteFooter from "@/app/command-palette/CommandPaletteFooter";
import CommandPaletteInput from "@/app/command-palette/CommandPaletteInput";
import useCommandPaletteSearch from "@/app/command-palette/useCommandPaletteSearch";

const CommandPaletteContent = (props: { onClose: () => void; open: boolean }): ReactElement => {
  const [query, setQuery] = useState("");
  const navigate = useNavigate();

  useEffect(() => {
    if (props.open === false) {
      setQuery("");
    }
  }, [props.open]);

  const { enabled, filteredPages, genes } = useCommandPaletteSearch(query);

  const onSelect = useCallback(
    (to: string): void => {
      props.onClose();
      navigate(to);
    },
    [navigate, props],
  );

  return (
    <>
      <CommandPaletteInput onChange={setQuery} value={query} />
      <CommandPaletteBody
        enabled={enabled}
        genes={genes}
        onSelect={onSelect}
        pages={filteredPages}
      />
      <CommandPaletteFooter />
    </>
  );
};

export default CommandPaletteContent;
