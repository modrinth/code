"use client";

import ComingSoon from "@/components/layout/ComingSoon";
import { useLocale } from "@/hooks/useLocale";

export default function OnlinePage() {
  const { dict } = useLocale();
  return <ComingSoon title={dict.comingSoon.onlineTitle} description={dict.comingSoon.onlineDesc} />;
}
