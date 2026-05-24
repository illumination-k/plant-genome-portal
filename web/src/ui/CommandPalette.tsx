import type { ReactElement } from "react";
import { useCallback } from "react";
import CommandPaletteDialog from "@/ui/CommandPaletteDialog";

const CommandPalette = (props: {
  onOpenChange: (open: boolean) => void;
  open: boolean;
}): ReactElement => {
  const onClose = useCallback((): void => {
    props.onOpenChange(false);
  }, [props]);

  return <CommandPaletteDialog onClose={onClose} onOpenChange={props.onOpenChange} open={props.open} />;
};

export default CommandPalette;
