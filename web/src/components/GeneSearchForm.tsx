import type { ReactElement } from "react";

const GeneSearchForm = (props: { searchText: string }): ReactElement => (
  <form action="/genes" className="mt-6 flex flex-col gap-3 sm:flex-row" method="get">
    <label className="sr-only" htmlFor="gene-search">
      Gene search
    </label>
    <input
      aria-label="Gene search"
      className="min-h-10 flex-1 rounded-md border border-zinc-300 bg-white px-3 text-sm text-zinc-950 outline-none transition placeholder:text-zinc-400 focus:border-emerald-700 focus:ring-2 focus:ring-emerald-100"
      defaultValue={props.searchText}
      id="gene-search"
      name="q"
      placeholder="Mp1g00070, gene symbol, locus tag"
      type="search"
    />
    <button
      className="min-h-10 rounded-md bg-emerald-700 px-4 text-sm font-semibold text-white transition hover:bg-emerald-800 focus:outline-none focus:ring-2 focus:ring-emerald-600 focus:ring-offset-2"
      type="submit"
    >
      Search
    </button>
  </form>
);

export default GeneSearchForm;
