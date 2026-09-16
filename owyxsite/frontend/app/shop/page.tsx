"use client";

import ComingSoon from "@/components/layout/ComingSoon";
import { useLocale } from "@/hooks/useLocale";

export default function ShopPage() {
  const { dict } = useLocale();
  return <ComingSoon title={dict.comingSoon.shopTitle} description={dict.comingSoon.shopDesc} />;
}
