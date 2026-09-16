"use client";

import ComingSoon from "@/components/layout/ComingSoon";
import { useLocale } from "@/hooks/useLocale";

export default function TokensPage() {
  const { dict } = useLocale();
  return <ComingSoon title={dict.comingSoon.tokensTitle} description={dict.comingSoon.tokensDesc} />;
}
