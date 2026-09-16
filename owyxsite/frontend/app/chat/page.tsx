"use client";

import ComingSoon from "@/components/layout/ComingSoon";
import { useLocale } from "@/hooks/useLocale";

export default function ChatPage() {
  const { dict } = useLocale();
  return <ComingSoon title={dict.comingSoon.chatTitle} description={dict.comingSoon.chatDesc} />;
}
