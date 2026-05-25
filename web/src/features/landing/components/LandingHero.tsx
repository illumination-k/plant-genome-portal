import type { ReactElement } from "react";

const LandingHero = (): ReactElement => (
  <div className="col-span-12 md:col-start-3 md:col-span-8">
    <p className="text-center font-mono text-[11px] uppercase tracking-[0.16em] text-text-subtle">
      <span className="italic">Marchantia polymorpha</span> · MpTak1 v7.1
    </p>
    <h1 className="mt-3 text-center text-[32px] font-bold leading-[40px] tracking-tight text-text">
      Search the plant genome portal
    </h1>
    <p className="mx-auto mt-3 max-w-[56ch] text-center text-[15px] leading-[22px] text-text-muted">
      Look up genes by accession, symbol, or locus tag. Browse functional annotation (GO, Pfam,
      InterPro, KEGG) and inspect gene structure on the genome.
    </p>
  </div>
);

export default LandingHero;
