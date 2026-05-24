const PALETTE_EVENT = "pgp:open-palette";

const isEditable = (target: EventTarget | null): boolean => {
  if (!(target instanceof globalThis.HTMLElement)) {
    return false;
  }
  if (target.isContentEditable) {
    return true;
  }
  const tag = target.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";
};

const isPaletteShortcut = (event: KeyboardEvent): boolean => {
  if (event.key !== "k" && event.key !== "K") {
    return false;
  }
  return event.metaKey || event.ctrlKey;
};

const openPalette = (): void => {
  globalThis.window.dispatchEvent(new globalThis.Event(PALETTE_EVENT));
};

const keyboardShortcuts = {
  PALETTE_EVENT,
  isEditable,
  isPaletteShortcut,
  openPalette,
};

export default keyboardShortcuts;
