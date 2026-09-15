"use client";

import Image from "next/image";

/** Owyx mark + word from brand/v2 — shared with the launcher. */
export default function Logo({
  size = 28,
  withWord = true,
  wordClassName = "text-lg",
}: {
  size?: number;
  withWord?: boolean;
  wordClassName?: string;
}) {
  return (
    <span className="inline-flex items-center gap-2">
      <Image
        src="/owyx-icon.svg"
        alt=""
        width={size}
        height={size}
        className="shrink-0"
        aria-hidden="true"
        unoptimized
      />
      {withWord && (
        <span className={`owyx-brand ${wordClassName}`}>
          <span className="owyx-brand-accent">owyx</span>
        </span>
      )}
    </span>
  );
}
