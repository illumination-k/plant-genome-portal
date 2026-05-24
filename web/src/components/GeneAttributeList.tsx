import GeneAttributeRow from "@/components/GeneAttributeRow";
import type { ReactElement } from "react";

const emptyCount = 0;

const GeneAttributeList = (props: { attributes: Record<string, string> }): ReactElement => {
  const attributes = Object.entries(props.attributes);

  if (attributes.length === emptyCount) {
    return <p className="mt-4 text-sm text-zinc-500">No attributes were included for this gene.</p>;
  }

  return (
    <dl className="mt-4 divide-y divide-zinc-200 text-sm">
      {attributes.map(([attributeKey, value]) => (
        <GeneAttributeRow attributeKey={attributeKey} key={attributeKey} value={value} />
      ))}
    </dl>
  );
};

export default GeneAttributeList;
