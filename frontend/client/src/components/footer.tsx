import { Link } from "wouter";
import { Shield } from "lucide-react";

interface FooterProps {
  /** Small mono label shown in the pill, e.g. "v2.0-stable", "docs", "pricing". */
  tag?: string;
}

const footerLinks = [
  { href: "/how-it-works", label: "How It Works" },
  { href: "/features", label: "Features" },
  { href: "/use-cases", label: "Use Cases" },
  { href: "/pricing", label: "Pricing" },
  { href: "/api", label: "API Docs" },
];

export function Footer({ tag = "v2.0-stable" }: FooterProps) {
  return (
    <footer className="relative border-t border-border/70 py-14 bg-background overflow-hidden">
      <div className="pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-primary/40 to-transparent" />
      <div className="max-w-6xl mx-auto px-4 flex flex-col items-center gap-6">
        <Link href="/" className="flex items-center gap-2.5">
          <span className="grid h-9 w-9 place-items-center rounded-xl bg-gradient-brand text-white shadow-[0_6px_18px_-6px_hsl(var(--brand-from)/0.7)]">
            <Shield className="h-5 w-5" />
          </span>
          <span className="text-xl font-bold font-display tracking-tight text-foreground">Verdyx</span>
        </Link>
        <nav className="flex flex-wrap justify-center gap-x-7 gap-y-2">
          {footerLinks.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              className="text-sm text-muted-foreground hover:text-foreground transition-colors"
            >
              {link.label}
            </Link>
          ))}
        </nav>
        <div className="text-muted-foreground/80 font-mono text-xs bg-surface px-4 py-1.5 rounded-full border border-border">
          verdyx // {tag}
        </div>
        <p className="text-muted-foreground/60 text-xs">© 2026 Verdyx. Developer First.</p>
      </div>
    </footer>
  );
}
