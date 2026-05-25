import { Dialog } from "@base-ui/react/dialog";
import type { ReactElement } from "react";
import CommandPalettePopupBody from "@/app/command-palette/CommandPalettePopupBody";

const renderBackdrop = <div className="fixed inset-0 z-40 bg-overlay" />;

const CommandPalettePortalContent = (props: {
  onClose: () => void;
  open: boolean;
}): ReactElement => (
  <Dialog.Portal>
    <Dialog.Backdrop render={renderBackdrop} />
    <CommandPalettePopupBody onClose={props.onClose} open={props.open} />
  </Dialog.Portal>
);

export default CommandPalettePortalContent;
