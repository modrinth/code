"use client";

import ComingSoon from "@/components/layout/ComingSoon";
import { useLocale } from "@/hooks/useLocale";

export default function ForumTopicPage() {
  const { dict } = useLocale();
  return <ComingSoon title={dict.comingSoon.forumTitle} description={dict.comingSoon.forumDesc} />;
}
