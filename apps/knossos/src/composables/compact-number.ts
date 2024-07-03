import { createFormatter, type Formatter } from '@vintl/compact-number'
import type { IntlController } from '@vintl/vintl/controller'

const formatters = new WeakMap<IntlController<any>, Formatter>()

export function useCompactNumber(): Formatter {
  const vintl = useVIntl()

  let formatter = formatters.get(vintl)

  if (formatter == null) {
    const formatterRef = computed(() => createFormatter(vintl.intl))
    formatter = (value, options) => formatterRef.value(value, options)
    formatters.set(vintl, formatter)
  }

  return formatter
}
