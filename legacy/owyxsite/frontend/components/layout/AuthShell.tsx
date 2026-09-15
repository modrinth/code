import Link from "next/link";
import Logo from "@/components/ui/Logo";

/** Clean, card-less auth layout — same brand language as the launcher. */
export default function AuthShell({
  title,
  subtitle,
  children,
  footer,
}: {
  title: string;
  subtitle?: string;
  children: React.ReactNode;
  footer?: React.ReactNode;
}) {
  return (
    <main id="main-content" className="min-h-screen flex flex-col">
      <div className="px-4 sm:px-6 py-5">
        <Link href="/" className="inline-flex items-center" aria-label="Owyx — на главную">
          <Logo size={26} wordClassName="text-xl" />
        </Link>
      </div>
      <div className="flex-1 grid place-items-center px-4 pb-20">
        <div className="w-full max-w-md min-w-0">
          <h1 className="font-display text-3xl sm:text-4xl font-bold tracking-[-0.04em]">{title}</h1>
          {subtitle && <p className="mt-2 text-muted leading-relaxed">{subtitle}</p>}
          <div className="mt-8">{children}</div>
          {footer && <div className="mt-6 text-sm text-muted">{footer}</div>}
        </div>
      </div>
    </main>
  );
}
