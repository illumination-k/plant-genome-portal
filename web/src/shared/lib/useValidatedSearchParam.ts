import type { GenericSchema, InferOutput } from "valibot";
import { useSearchParams } from "react-router";
import { safeParse } from "valibot";

/**
 * Validate a single URL search-param via valibot. `URLSearchParams.get(key)`
 * may return null when the key is absent; in that case (or any failing parse)
 * the fallback is returned. Consumers always see a fully-typed value.
 */
const useValidatedSearchParam = <Schema extends GenericSchema<string, unknown>>(
  key: string,
  schema: Schema,
  fallback: InferOutput<Schema>,
): InferOutput<Schema> => {
  const [searchParams] = useSearchParams();
  const raw = searchParams.get(key);
  const result = safeParse(schema, raw);
  if (result.success) {
    return result.output;
  }
  return fallback;
};

export default useValidatedSearchParam;
