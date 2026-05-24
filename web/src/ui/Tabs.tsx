import { Tabs as BaseTabs } from "@base-ui/react/tabs";
import type { ReactElement, ReactNode } from "react";

type Tab = {
  label: string;
  panel: ReactNode;
  value: string;
};

type Props = {
  ariaLabel: string;
  onValueChange: (value: string) => void;
  tabs: Tab[];
  value: string;
};

const tabClass =
  "relative cursor-pointer border-b-2 border-transparent px-3 py-2 text-sm font-medium text-text-muted transition data-[selected]:border-primary-700 data-[selected]:text-text hover:text-text focus-visible:outline-none";

/*
 * Children (the tab label) are injected by base-ui via cloneElement, but the
 * linter only sees the static JSX, so an aria-label keeps it satisfied. The
 * real text appears via the Tabs.Tab children.
 */
const renderTabButton = <button aria-label="tab" className={tabClass} type="button" />;
const renderListNav = <nav className="flex gap-1 border-b border-border-subtle" />;
const renderPanelDiv = <div className="pt-6 outline-none" />;

const renderTab = (tab: Tab): ReactElement => (
  <BaseTabs.Tab key={tab.value} render={renderTabButton} value={tab.value}>
    {tab.label}
  </BaseTabs.Tab>
);

const renderPanel = (tab: Tab): ReactElement => (
  <BaseTabs.Panel key={tab.value} render={renderPanelDiv} value={tab.value}>
    {tab.panel}
  </BaseTabs.Panel>
);

const Tabs = (props: Props): ReactElement => (
  <BaseTabs.Root onValueChange={props.onValueChange} value={props.value}>
    <BaseTabs.List aria-label={props.ariaLabel} render={renderListNav}>
      {props.tabs.map(renderTab)}
    </BaseTabs.List>
    {props.tabs.map(renderPanel)}
  </BaseTabs.Root>
);

export default Tabs;
