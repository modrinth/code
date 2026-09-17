"use client";

/** Owyx™ crystal mark + word — fill follows active theme accent via currentColor. */
export default function Logo({
  size = 28,
  withWord = true,
  wordClassName = "text-lg",
  withTm = true,
}: {
  size?: number;
  withWord?: boolean;
  wordClassName?: string;
  /** Show ™ after the wordmark (header / primary brand surfaces). */
  withTm?: boolean;
}) {
  return (
    <span className="inline-flex items-center gap-2 text-accent">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 64 64"
        width={size}
        height={size}
        className="shrink-0"
        aria-hidden="true"
        fill="none"
      >
        <path d="M32 4 L56 18 L32 32 Z" fill="currentColor" fillOpacity="0.95" />
        <path d="M8 18 L32 4 L32 32 Z" fill="currentColor" fillOpacity="0.75" />
        <path d="M56 18 L56 46 L32 32 Z" fill="currentColor" fillOpacity="0.55" />
        <path d="M8 46 L8 18 L32 32 Z" fill="currentColor" fillOpacity="0.42" />
        <path d="M56 46 L32 60 L32 32 Z" fill="currentColor" fillOpacity="0.3" />
        <path d="M32 60 L8 46 L32 32 Z" fill="currentColor" fillOpacity="0.18" />
        <polygon
          points="32,4 56,18 56,46 32,60 8,46 8,18"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinejoin="round"
          strokeLinecap="round"
        />
        <circle cx="32" cy="32" r="1.6" fill="currentColor" />
        <circle cx="32" cy="32" r="0.8" fill="var(--accent-hi)" />
      </svg>
      {withWord && (
        <span className={`owyx-brand ${wordClassName}`}>
          <span className="owyx-brand-accent">owyx</span>
          {withTm && (
            <sup className="ml-0.5 text-[0.45em] font-semibold tracking-normal text-accent/80 align-super">
              ™
            </sup>
          )}
        </span>
      )}
    </span>
  );
}
