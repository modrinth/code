"use client";

import { useId } from "react";

/** Owyx hex/crystal mark — shared with the launcher brand (cyan → violet). */
export default function Logo({
  size = 28,
  withWord = true,
  wordClassName = "text-lg",
}: {
  size?: number;
  withWord?: boolean;
  wordClassName?: string;
}) {
  const gid = `owyxHex-${useId().replace(/:/g, "")}`;
  return (
    <span className="inline-flex items-center gap-2">
      <svg
        width={size}
        height={size}
        viewBox="0 0 32 32"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        aria-hidden="true"
      >
        <path
          d="M16 2L2 9.5V22.5L16 30L30 22.5V9.5L16 2Z"
          stroke={`url(#${gid})`}
          strokeLinejoin="round"
          strokeWidth={2}
        />
        <path
          d="M16 8L8 12.5V20L16 24L24 20V12.5L16 8Z"
          stroke="var(--accent, #00e5ff)"
          strokeLinejoin="round"
          strokeWidth={1.5}
        />
        <defs>
          <linearGradient id={gid} x1="16" y1="2" x2="16" y2="30" gradientUnits="userSpaceOnUse">
            <stop stopColor="var(--accent-violet, #8b5cf6)" />
            <stop offset="1" stopColor="var(--accent, #00e5ff)" />
          </linearGradient>
        </defs>
      </svg>
      {withWord && (
        <span className={`owyx-brand ${wordClassName}`}>
          <span className="owyx-brand-accent">owyx</span>
        </span>
      )}
    </span>
  );
}
