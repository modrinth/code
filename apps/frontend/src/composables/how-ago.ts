import {createFormatter, type FormatOptions, type Formatter} from "@vintl/how-ago";
import type { IntlController } from "@vintl/vintl/controller";

const formatters = new WeakMap<IntlController<any>, Formatter>();

export function useRelativeTime(): Formatter {
  const vintl = useVIntl();

  let formatter = formatters.get(vintl);

  if (formatter == null) {
    const formatterRef = computed(() => createFormatter(vintl.intl));
    const defaultOptions: FormatOptions = { roundingMode: "halfExpand" as const };

    formatter = (value, options) => formatterRef.value(value, {...options, ...defaultOptions});
    formatters.set(vintl, formatter);
  }

  return formatter;
}
