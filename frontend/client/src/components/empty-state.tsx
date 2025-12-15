import { Button } from "@/components/ui/button";
import { Card, CardContent, CardFooter } from "@/components/ui/card";
import { LucideIcon } from "lucide-react";

interface EmptyStateProps {
  icon: LucideIcon;
  title: string;
  description: string;
  actionLabel?: string;
  onAction?: () => void;
}

export function EmptyState({
  icon: Icon,
  title,
  description,
  actionLabel,
  onAction,
}: EmptyStateProps) {
  return (
    <Card className="glassmorphism border-primary/20">
      <CardContent className="flex flex-col items-center justify-center py-12 text-center">
        <div className="p-4 bg-muted/20 rounded-full mb-4">
          <Icon className="h-12 w-12 text-muted-foreground" />
        </div>
        <h3 className="text-xl font-semibold mb-2">{title}</h3>
        <p className="text-muted-foreground max-w-md">{description}</p>
      </CardContent>
      {actionLabel && onAction && (
        <CardFooter className="flex justify-center">
          <Button onClick={onAction} className="glow-effect">
            {actionLabel}
          </Button>
        </CardFooter>
      )}
    </Card>
  );
}

export function InlineEmptyState({
  icon: Icon,
  message,
}: {
  icon: LucideIcon;
  message: string;
}) {
  return (
    <div className="flex flex-col items-center justify-center py-8 text-center">
      <Icon className="h-10 w-10 text-muted-foreground mb-3" />
      <p className="text-muted-foreground">{message}</p>
    </div>
  );
}