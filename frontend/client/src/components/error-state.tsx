import { Button } from "@/components/ui/button";
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { AlertCircle, RefreshCw } from "lucide-react";

interface ErrorStateProps {
  title?: string;
  message?: string;
  onRetry?: () => void;
  showRetry?: boolean;
}

export function ErrorState({
  title = "Something went wrong",
  message = "We encountered an error while loading this content. Please try again.",
  onRetry,
  showRetry = true,
}: ErrorStateProps) {
  return (
    <Card className="glassmorphism border-destructive/20">
      <CardHeader>
        <div className="flex items-center gap-3">
          <div className="p-2 bg-destructive/20 rounded-full">
            <AlertCircle className="h-6 w-6 text-destructive" />
          </div>
          <CardTitle className="text-destructive">{title}</CardTitle>
        </div>
      </CardHeader>
      <CardContent>
        <p className="text-muted-foreground">{message}</p>
      </CardContent>
      {showRetry && onRetry && (
        <CardFooter>
          <Button onClick={onRetry} variant="outline" className="w-full">
            <RefreshCw className="mr-2 h-4 w-4" />
            Try Again
          </Button>
        </CardFooter>
      )}
    </Card>
  );
}

export function InlineErrorState({
  message = "Failed to load data",
  onRetry,
}: {
  message?: string;
  onRetry?: () => void;
}) {
  return (
    <div className="flex items-center justify-between p-4 bg-destructive/10 border border-destructive/20 rounded-lg">
      <div className="flex items-center gap-3">
        <AlertCircle className="h-5 w-5 text-destructive" />
        <span className="text-sm text-destructive">{message}</span>
      </div>
      {onRetry && (
        <Button onClick={onRetry} variant="ghost" size="sm">
          <RefreshCw className="h-4 w-4" />
        </Button>
      )}
    </div>
  );
}
