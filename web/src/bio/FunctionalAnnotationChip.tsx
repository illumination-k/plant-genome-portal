import type { ReactElement } from "react";

const className =
  "inline-flex max-w-full items-center gap-1.5 truncate rounded-full border border-border-subtle bg-surface-muted px-2 py-0.5 text-[12px] hover:border-border";

const renderName = (name: string): ReactElement | false => {
  if (name) {
    return <span className="text-text-muted">{name}</span>;
  }
  return false;
};

const renderLinked = (props: { href: string; id: string; name: string }): ReactElement => (
  <a className={className} href={props.href} rel="noreferrer" target="_blank">
    <span className="font-mono">{props.id}</span>
    {renderName(props.name)}
  </a>
);

const renderPlain = (props: { id: string; name: string }): ReactElement => (
  <span className={className}>
    <span className="font-mono">{props.id}</span>
    {renderName(props.name)}
  </span>
);

const FunctionalAnnotationChip = (props: {
  href: string;
  id: string;
  name: string;
}): ReactElement => {
  if (props.href) {
    return renderLinked(props);
  }
  return renderPlain({ id: props.id, name: props.name });
};

export default FunctionalAnnotationChip;
