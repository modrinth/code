"use client";

import { useLocale } from "@/hooks/useLocale";

export default function SkipLink() {
  const { dict } = useLocale();

  return (
    <a href="#main-content" className="skip-link">
      {dict.common.skipToContent}
    </a>
  );
}
