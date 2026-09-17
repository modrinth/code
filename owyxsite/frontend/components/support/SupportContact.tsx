"use client";

import { useEffect, useId, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { useLocale } from "@/hooks/useLocale";

/** New GitHub issue on the Owyx launcher/site monorepo. */
export const OWYX_SUPPORT_ISSUE_URL =
  "https://github.com/ebluffy/Owyx/issues/new?template=blank&title=%5BSupport%5D%20";

export function SupportContactButton({
  className = "btn btn-ghost",
  children,
}: {
  className?: string;
  children?: ReactNode;
}) {
  const [open, setOpen] = useState(false);
  const { locale } = useLocale();
  const en = locale === "en_US";
  const label = children ?? (en ? "Contact support" : "Связаться с поддержкой");

  return (
    <>
      <button type="button" className={className} onClick={() => setOpen(true)}>
        {label}
      </button>
      {open && <SupportContactModal onClose={() => setOpen(false)} />}
    </>
  );
}

export function SupportContactModal({ onClose }: { onClose: () => void }) {
  const { locale } = useLocale();
  const en = locale === "en_US";
  const titleId = useId();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
    const prev = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => {
      document.body.style.overflow = prev;
      window.removeEventListener("keydown", onKey);
    };
  }, [onClose]);

  if (!mounted) return null;

  return createPortal(
    <div
      className="fixed inset-0 z-[80] flex items-center justify-center p-4"
      role="presentation"
      onClick={onClose}
    >
      <div className="absolute inset-0 bg-black/65 backdrop-blur-sm" />
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="panel relative z-10 w-full max-w-md p-6 shadow-2xl fade-up"
        onClick={(e) => e.stopPropagation()}
      >
        <p className="text-xs uppercase tracking-[0.14em] text-accent/80">
          {en ? "Support" : "Поддержка"}
        </p>
        <h2 id={titleId} className="font-display mt-2 text-xl font-bold tracking-tight text-text">
          {en ? "Live support isn’t ready yet" : "Живой поддержки пока нет"}
        </h2>
        <p className="mt-3 text-sm leading-relaxed text-muted">
          {en
            ? "We don’t have an in-app support chat yet. You can open a GitHub issue — we watch the tracker regularly."
            : "Чата поддержки на сайте пока нет. Можно создать issue на GitHub — мы смотрим трекер регулярно."}
        </p>
        <div className="mt-6 flex flex-wrap gap-2 justify-end">
          <button type="button" className="btn btn-ghost" onClick={onClose}>
            {en ? "Close" : "Закрыть"}
          </button>
          <a
            href={OWYX_SUPPORT_ISSUE_URL}
            target="_blank"
            rel="noopener noreferrer"
            className="btn btn-primary"
          >
            {en ? "Open GitHub issue" : "Создать issue на GitHub"}
          </a>
        </div>
      </div>
    </div>,
    document.body,
  );
}
