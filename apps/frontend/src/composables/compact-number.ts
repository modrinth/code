const formatters = new WeakMap<object, Intl.NumberFormat>();

export function useCompactNumber(truncate = false, fractionDigits = 2, locale?: string) {
  const context = {};

  let formatter = formatters.get(context);

  if (!formatter) {
    formatter = new Intl.NumberFormat(locale, {
      notation: "compact",
      maximumFractionDigits: fractionDigits,
    });
    formatters.set(context, formatter);
  }

  function format(value: number): string {
    let formattedValue = value;
    if (truncate) {
      const scale = Math.pow(10, fractionDigits);
      formattedValue = Math.floor(value * scale) / scale;
    }
    return formatter!.format(formattedValue);
  }

  return format;
}
