import { Link } from "wouter";

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
    <footer className="border-t border-border py-12 bg-background">
      <div className="max-w-6xl mx-auto px-4 flex flex-col items-center gap-6">
        <nav className="flex flex-wrap justify-center gap-x-6 gap-y-2">
          {footerLinks.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              className="text-sm text-muted-foreground hover:text-primary transition-colors"
            >
              {link.label}
            </Link>
          ))}
        </nav>
        <div className="text-muted-foreground/80 font-mono text-sm bg-surface px-4 py-1 rounded-full border border-border">
          verdyx // {tag}
        </div>
        <p className="text-muted-foreground/60 text-xs">© 2026 Verdyx. Developer First.</p>
      </div>
    </footer>
  );
}
