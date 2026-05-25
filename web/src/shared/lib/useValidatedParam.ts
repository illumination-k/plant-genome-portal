import type { GenericSchema, InferOutput } from "valibot";
import { useParams } from "react-router";
import { safeParse } from "valibot";

/**
 * Validate a single URL path-param via valibot. `useParams()` may return
 * undefined when the route hasn't matched; push the raw value through the
 * schema so consumers always work with a fully-typed value.
 */
const useValidatedParam = <Schema extends GenericSchema<string, unknown>>(
  key: string,
  schema: Schema,
  fallback: InferOutput<Schema>,
): InferOutput<Schema> => {
  const params = useParams();
  const raw = params[key];
  const result = safeParse(schema, raw);
  if (result.success) {
    return result.output;
  }
  return fallback;
};

export default useValidatedParam;
