/* oxlint-disable import/no-named-export, import/only-export-components, react/only-export-components, react-perf/jsx-no-new-function-as-prop, eslint/no-use-before-define, eslint/no-ternary, react/jsx-max-depth, eslint/max-lines-per-function */
import type { HomologySearchMethod } from "@/api/client/types.gen";
import type { ChangeEvent, FormEvent, ReactElement } from "react";
import {
  blastnTasks,
  blastpTasks,
  methodDefaults,
  methodDescription,
  methodLabel,
} from "@/features/tools/blast/lib/blastConfig";
import type { BlastForm } from "@/features/tools/blast/lib/blastConfig";

type BlastFormPanelProps = {
  form: BlastForm;
  isRunning: boolean;
  isSubmitting: boolean;
  onChange: (form: BlastForm) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => Promise<void>;
};

const BlastFormPanel = (props: BlastFormPanelProps): ReactElement => {
  const taskOptions = props.form.method === "blastp" ? blastpTasks : blastnTasks;

  const updateField = (
    event: ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>,
  ): void => {
    props.onChange({ ...props.form, [event.target.name]: event.target.value });
  };

  const updateMethod = (method: HomologySearchMethod): void => {
    if (props.form.method === method) {
      return;
    }
    props.onChange({ ...props.form, method, ...methodDefaults(method) });
  };

  return (
    <form className="mt-6 grid grid-cols-12 gap-4" onSubmit={props.onSubmit}>
      <fieldset className="col-span-12 flex flex-wrap items-center gap-3">
        <legend className="sr-only">BLAST method</legend>
        <span className="text-xs font-medium uppercase text-text-subtle">Method</span>
        <MethodRadio
          checked={props.form.method === "blastn"}
          label="blastn (nucleotide)"
          onChange={() => updateMethod("blastn")}
        />
        <MethodRadio
          checked={props.form.method === "blastp"}
          label="blastp (protein)"
          onChange={() => updateMethod("blastp")}
        />
      </fieldset>

      <label className="col-span-12 flex flex-col gap-1 md:col-span-5">
        <span className="text-xs font-medium uppercase text-text-subtle">Assembly accession</span>
        <input
          aria-label="Assembly accession"
          className="min-h-10 rounded-md border border-border bg-surface px-3 font-mono text-sm text-text outline-none transition placeholder:text-text-subtle focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
          name="assemblyAccession"
          onChange={updateField}
          required
          value={props.form.assemblyAccession}
        />
      </label>

      <label className="col-span-12 flex flex-col gap-1 sm:col-span-4 md:col-span-3">
        <span className="text-xs font-medium uppercase text-text-subtle">Task</span>
        <select
          aria-label="BLAST task"
          className="min-h-10 rounded-md border border-border bg-surface px-3 text-sm text-text outline-none transition focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
          name="task"
          onChange={updateField}
          value={props.form.task}
        >
          {taskOptions.map((task) => (
            <option key={task} value={task}>
              {task}
            </option>
          ))}
        </select>
      </label>

      <label className="col-span-6 flex flex-col gap-1 sm:col-span-4 md:col-span-2">
        <span className="text-xs font-medium uppercase text-text-subtle">E-value</span>
        <input
          aria-label="E-value"
          className="min-h-10 rounded-md border border-border bg-surface px-3 text-sm text-text outline-none transition focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
          min="0.0000000001"
          name="evalue"
          onChange={updateField}
          required
          step="any"
          type="number"
          value={props.form.evalue}
        />
      </label>

      <label className="col-span-6 flex flex-col gap-1 sm:col-span-4 md:col-span-2">
        <span className="text-xs font-medium uppercase text-text-subtle">Max hits</span>
        <input
          aria-label="Maximum hits"
          className="min-h-10 rounded-md border border-border bg-surface px-3 text-sm text-text outline-none transition focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
          min="1"
          name="maxTargetSeqs"
          onChange={updateField}
          required
          step="1"
          type="number"
          value={props.form.maxTargetSeqs}
        />
      </label>

      <label className="col-span-12 flex flex-col gap-1">
        <span className="text-xs font-medium uppercase text-text-subtle">Query</span>
        <textarea
          aria-label="Query sequence"
          className="min-h-48 resize-y rounded-md border border-border bg-surface px-3 py-3 font-mono text-sm leading-6 text-text outline-none transition placeholder:text-text-subtle focus:border-primary-700 focus:ring-2 focus:ring-primary-100"
          name="query"
          onChange={updateField}
          required
          spellCheck={false}
          value={props.form.query}
        />
      </label>

      <div className="col-span-12 flex flex-wrap items-center gap-3">
        <button
          className="min-h-10 rounded-md bg-primary-700 px-4 text-sm font-semibold text-white transition hover:bg-primary-800 disabled:cursor-not-allowed disabled:bg-text-disabled"
          disabled={props.isSubmitting || props.isRunning}
          type="submit"
        >
          {props.isSubmitting || props.isRunning
            ? "Running"
            : `Run ${methodLabel(props.form.method)}`}
        </button>
      </div>
    </form>
  );
};

const MethodRadio = (props: {
  checked: boolean;
  label: string;
  onChange: () => void;
}): ReactElement => (
  <label className="inline-flex items-center gap-2 text-sm text-text">
    <input
      aria-label={props.label}
      checked={props.checked}
      className="h-4 w-4 accent-primary-700"
      name="method"
      onChange={props.onChange}
      type="radio"
    />
    {props.label}
  </label>
);

export { BlastFormPanel, methodDescription, methodLabel };
