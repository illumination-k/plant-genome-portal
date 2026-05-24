import type { ReactElement } from "react";

const GeneSearchForm = (props: { searchText: string }): ReactElement => (
  <form action="/genes" className="mt-6 flex flex-col gap-3 sm:flex-row" method="get">
    <label className="sr-only" htmlFor="gene-search">
      Gene search
    </label>
    <input
      aria-label="Gene search"
      className="min-h-10 flex-1 rounded-md border border-border bg-surface px-3 text-sm text-text outline-none transition placeholder:text-text-subtle focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
      defaultValue={props.searchText}
      id="gene-search"
      name="q"
      placeholder="Mp1g00070, gene symbol, locus tag"
      type="search"
    />
    <button
      className="min-h-10 rounded-md bg-primary-700 px-4 text-sm font-semibold text-white transition hover:bg-primary-800 focus:outline-none focus:ring-2 focus:ring-primary-600 focus:ring-offset-2"
      type="submit"
    >
      Search
    </button>
  </form>
);

export default GeneSearchForm;
