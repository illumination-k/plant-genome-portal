import type { FormEvent, ReactElement } from "react";
import { useCallback } from "react";
import { useNavigate } from "react-router";

const buildSearchUrl = (raw: string): string => {
  const trimmed = raw.trim();
  if (trimmed === "") {
    return "/genes";
  }
  return `/genes?q=${encodeURIComponent(trimmed)}`;
};

const readQuery = (form: HTMLFormElement): string => {
  const input = form.elements.namedItem("q");
  if (input instanceof HTMLInputElement) {
    return input.value;
  }
  return "";
};

const LandingSearchForm = (): ReactElement => {
  const navigate = useNavigate();

  const onSubmit = useCallback(
    (event: FormEvent<HTMLFormElement>): void => {
      event.preventDefault();
      navigate(buildSearchUrl(readQuery(event.currentTarget)));
    },
    [navigate],
  );

  return (
    <form className="mx-auto mt-8 flex max-w-2xl gap-2" onSubmit={onSubmit}>
      <input
        aria-label="Search genes"
        className="h-10 min-w-0 flex-1 rounded-md border border-border bg-surface px-3 font-mono text-sm text-text outline-none transition placeholder:text-text-subtle hover:border-border-strong focus:border-primary-500 focus:ring-3 focus:ring-primary-100"
        id="landing-q"
        name="q"
        placeholder="Mp1g00010, MpARF1, or locus tag…"
        type="search"
      />
      <button
        className="h-10 rounded-md bg-primary-700 px-5 text-sm font-semibold text-white transition hover:bg-primary-800 focus-visible:ring-3 focus-visible:ring-primary-200"
        type="submit"
      >
        Search
      </button>
    </form>
  );
};

export default LandingSearchForm;
