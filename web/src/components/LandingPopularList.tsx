import type { ReactElement } from "react";
import LandingPopularRow from "@/components/LandingPopularRow";

const examples = [
  { id: "Mp1g00010", note: "first gene on Chr1" },
  { id: "Mp3g18560", note: "MpARF1 · auxin response" },
  { id: "Mp6g08920", note: "MpYUC1 · IAA biosynthesis" },
];

const LandingPopularList = (): ReactElement => (
  <div className="col-span-12 md:col-span-4">
    <h2 className="text-[11px] font-semibold uppercase tracking-[0.08em] text-text-subtle">
      Popular genes
    </h2>
    <ul className="mt-3 list-none divide-y divide-border-subtle rounded-md border border-border-subtle bg-surface p-0">
      {examples.map((example) => (
        <LandingPopularRow id={example.id} key={example.id} note={example.note} />
      ))}
    </ul>
  </div>
);

export default LandingPopularList;
