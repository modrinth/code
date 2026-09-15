/** Brand v2 token strip — matches brand/v2/preview.html §06 */
const TOKENS = [
  { label: "Cyan solid", value: "#00e5ff" },
  { label: "Cyan gradient", value: "#bff8ff → #00e5ff → #0b7a96" },
  { label: "Ink solid", value: "#0e0e14" },
  { label: "Slate solid", value: "#64748b" },
  { label: "Icon highlight", value: "#9ff7ff" },
  { label: "Icon core", value: "#bff8ff" },
] as const;

export default function BrandPalette() {
  return (
    <aside
      className="pointer-events-none fixed bottom-4 right-4 z-0 hidden lg:flex max-w-[min(22rem,42vw)] flex-col gap-2 rounded-xl border border-line/80 bg-panel/75 backdrop-blur-sm p-3 shadow-[0_12px_40px_-20px_rgba(0,229,255,0.35)]"
      aria-hidden="true"
    >
      <p className="font-mono text-[10px] uppercase tracking-[0.12em] text-muted">
        Токены v2
      </p>
      <div className="flex flex-wrap gap-1.5">
        {TOKENS.map((t) => (
          <span
            key={t.label}
            className="font-mono text-[10px] leading-snug px-2 py-1 rounded-md border border-line bg-panel-2 text-muted"
          >
            <strong className="text-text font-medium">{t.label}</strong>
            <span className="text-muted"> · </span>
            {t.value}
          </span>
        ))}
      </div>
    </aside>
  );
}
