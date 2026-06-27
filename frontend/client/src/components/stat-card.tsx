import { type LucideIcon, TrendingUp, TrendingDown } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";

type Accent = "primary" | "accent" | "warning" | "destructive";

const accentMap: Record<Accent, { text: string; ring: string; iconBg: string }> = {
  primary: { text: "text-primary", ring: "border-primary/20", iconBg: "bg-primary/10 text-primary" },
  accent: { text: "text-accent", ring: "border-accent/20", iconBg: "bg-accent/10 text-accent" },
  warning: { text: "text-warning", ring: "border-warning/20", iconBg: "bg-warning/10 text-warning" },
  destructive: { text: "text-destructive", ring: "border-destructive/20", iconBg: "bg-destructive/10 text-destructive" },
};

interface StatCardProps {
  label: string;
  value: string | number;
  icon: LucideIcon;
  accent?: Accent;
  /** Optional supporting line under the value. */
  hint?: string;
  /** Optional percentage trend, e.g. +12.3. Positive renders green-up, negative red-down. */
  trend?: number;
  loading?: boolean;
}

/**
 * A uniform KPI card. Every instance shares the exact same internal structure
 * and spacing so a row of them is perfectly symmetrical regardless of content.
 */
export function StatCard({ label, value, icon: Icon, accent = "primary", hint, trend, loading }: StatCardProps) {
  const colors = accentMap[accent];

  return (
    <Card className={cn("glassmorphism", colors.ring)}>
      <CardContent className="p-6">
        <div className="flex items-start justify-between">
          <div className="space-y-1 min-w-0">
            <p className="text-sm font-medium text-muted-foreground truncate">{label}</p>
            {loading ? (
              <Skeleton className="h-8 w-24" />
            ) : (
              <p className="text-3xl font-bold tracking-tight tabular-nums">{value}</p>
            )}
          </div>
          <div className={cn("flex h-10 w-10 shrink-0 items-center justify-center rounded-lg", colors.iconBg)}>
            <Icon className="h-5 w-5" />
          </div>
        </div>

        <div className="mt-4 h-5">
          {loading ? (
            <Skeleton className="h-4 w-28" />
          ) : trend !== undefined ? (
            <div
              className={cn(
                "flex items-center gap-1 text-xs font-medium",
                trend >= 0 ? "text-success" : "text-destructive"
              )}
            >
              {trend >= 0 ? <TrendingUp className="h-3 w-3" /> : <TrendingDown className="h-3 w-3" />}
              <span>{trend >= 0 ? "+" : ""}{trend}% from last week</span>
            </div>
          ) : hint ? (
            <p className="text-xs text-muted-foreground">{hint}</p>
          ) : null}
        </div>
      </CardContent>
    </Card>
  );
}
