"use client";

import ComingSoon from "@/components/layout/ComingSoon";
import { useLocale } from "@/hooks/useLocale";

/** Parked: public marketing is launcher + site; /servers redirects to /download. */
export default function ServersPage() {
  const { dict } = useLocale();
  return (
    <ComingSoon
      title={dict.comingSoon.title}
      description={dict.comingSoon.description}
    />
  );
}
