import type { ReactElement } from "react";
import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router";
import CommandPaletteBody from "@/ui/CommandPaletteBody";
import CommandPaletteFooter from "@/ui/CommandPaletteFooter";
import CommandPaletteInput from "@/ui/CommandPaletteInput";
import useCommandPaletteSearch from "@/ui/useCommandPaletteSearch";

const CommandPaletteContent = (props: {
  onClose: () => void;
  open: boolean;
}): ReactElement => {
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
