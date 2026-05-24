import { Dialog } from "@base-ui/react/dialog";
import type { ReactElement } from "react";
import CommandPalettePortalContent from "@/ui/CommandPalettePortalContent";

const CommandPaletteDialog = (props: {
  onClose: () => void;
  onOpenChange: (open: boolean) => void;
  open: boolean;
}): ReactElement => (
  <Dialog.Root onOpenChange={props.onOpenChange} open={props.open}>
    <CommandPalettePortalContent onClose={props.onClose} open={props.open} />
  </Dialog.Root>
);

export default CommandPaletteDialog;
