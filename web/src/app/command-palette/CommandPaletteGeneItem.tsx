import type { Gene } from "@/api/client/types.gen";
import type { ReactElement } from "react";
import { useCallback } from "react";
import CommandPaletteItem from "@/app/command-palette/CommandPaletteItem";

const CommandPaletteGeneItem = (props: {
  gene: Gene;
  onSelect: (to: string) => void;
}): ReactElement => {
  const onSelect = useCallback((): void => {
    props.onSelect(`/genes/${props.gene.id}`);
  }, [props]);

  const detail = props.gene.symbol ?? props.gene.sequence_name;

  return <CommandPaletteItem detail={detail} label={props.gene.id} mono onSelect={onSelect} />;
};

export default CommandPaletteGeneItem;
