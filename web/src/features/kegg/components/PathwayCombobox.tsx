import type { KeggPathwaySummary } from "@/api/client/types.gen";
import type { ChangeEvent, FormEvent, ReactElement } from "react";
import { useCallback } from "react";

const pathwayOptionLimit = 50;
const empty = 0;

const pathwayLabel = (pathway: KeggPathwaySummary): string => {
  const { id, name } = pathway.pathway;
  if (!name) {
    return id;
  }
  return `${id} · ${name}`;
};

const pathwayMatches = (pathway: KeggPathwaySummary, query: string): boolean => {
  const normalized = query.trim().toLowerCase();
  if (normalized === "") {
    return true;
  }
  return pathwayLabel(pathway).toLowerCase().includes(normalized);
};

const optionValue = (pathway: KeggPathwaySummary): string => pathway.pathway.id;

const PathwayDatalist = (props: { options: KeggPathwaySummary[] }): ReactElement => (
  <datalist id="pathway-options">
    {props.options.map((pathway) => (
      <option key={pathway.pathway.id} value={optionValue(pathway)}>
        {pathwayLabel(pathway)}
      </option>
    ))}
  </datalist>
);

const PathwayComboboxActions = (props: { optionCount: number }): ReactElement => (
  <div className="mt-3 flex items-center justify-between gap-3">
    <p className="text-[12px] text-text-subtle">
      {props.optionCount} matching pathways shown in the combobox
    </p>
    <button
      className="h-9 rounded-md bg-primary-700 px-4 text-sm font-semibold text-text-inverse transition hover:bg-primary-800 focus-visible:ring-3 focus-visible:ring-primary-200"
      type="submit"
    >
      Open pathway
    </button>
  </div>
);

const PathwayCombobox = (props: {
  onQueryChange: (value: string) => void;
  onSubmit: (value: string) => void;
  pathways: KeggPathwaySummary[];
  query: string;
}): ReactElement => {
  const options = props.pathways
    .filter((pathway) => pathwayMatches(pathway, props.query))
    .slice(empty, pathwayOptionLimit);
  const optionCount = options.length;

  const onChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>): void => {
      props.onQueryChange(event.target.value);
    },
    [props],
  );

  const onSubmit = useCallback(
    (event: FormEvent<HTMLFormElement>): void => {
      event.preventDefault();
      props.onSubmit(props.query);
    },
    [props],
  );

  return (
    <form className="rounded-lg border border-border-subtle bg-surface p-4" onSubmit={onSubmit}>
      <label className="flex flex-col gap-2 text-sm font-medium text-text" htmlFor="pathway-id">
        Pathway
        <input
          aria-label="Pathway"
          className="h-10 rounded-md border border-border bg-surface px-3 font-mono text-sm text-text outline-none transition placeholder:text-text-subtle hover:border-border-strong focus:border-primary-500 focus:ring-3 focus:ring-primary-100"
          id="pathway-id"
          list="pathway-options"
          onChange={onChange}
          placeholder="map00010"
          type="search"
          value={props.query}
        />
      </label>
      <PathwayDatalist options={options} />
      <PathwayComboboxActions optionCount={optionCount} />
    </form>
  );
};

export default PathwayCombobox;
