import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Bot, UserCheck, Brain, Fingerprint } from "lucide-react";
import type { SecurityEngine } from "@shared/schema";

interface EngineCardProps {
  engine: SecurityEngine;
}

export function EngineCard({ engine }: EngineCardProps) {
  const getEngineIcon = (type: string) => {
    switch (type) {
      case "automated":
        return <Bot className="w-6 h-6 text-primary" />;
      case "human":
        return <UserCheck className="w-6 h-6 text-accent" />;
      case "ml":
        return <Brain className="w-6 h-6 text-secondary" />;
      case "signature":
        return <Fingerprint className="w-6 h-6 text-primary" />;
      default:
        return <Bot className="w-6 h-6 text-primary" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case "online":
        return "text-accent";
      case "offline":
        return "text-muted-foreground";
      case "busy":
        return "text-secondary";
      default:
        return "text-muted-foreground";
    }
  };

  return (
    <Card className="glassmorphism border-primary/20 hover:border-primary/40 transition-colors group hover:transform hover:-translate-y-1 duration-300">
      <CardContent className="p-6">
        <div className="flex items-center space-x-3 mb-4">
          <div className="w-12 h-12 bg-primary/20 rounded-lg flex items-center justify-center">
            {getEngineIcon(engine.type)}
          </div>
          <div>
            <div className="font-semibold" data-testid={`engine-name-${engine.id}`}>
              {engine.name}
            </div>
            <div className="text-sm text-muted-foreground capitalize">
              {engine.type}
            </div>
          </div>
        </div>
        
        <div className="space-y-2 mb-4">
          <div className="flex justify-between text-sm">
            <span>Accuracy</span>
            <span className="text-accent font-medium" data-testid={`accuracy-${engine.id}`}>
              {engine.accuracy}%
            </span>
          </div>
          <div className="flex justify-between text-sm">
            <span>Analyses</span>
            <span className="text-primary font-medium" data-testid={`analyses-${engine.id}`}>
              {(engine.totalAnalyses || 0).toLocaleString()}
            </span>
          </div>
          <div className="flex justify-between text-sm">
            <span>Stake</span>
            <span className="text-secondary font-medium" data-testid={`stake-${engine.id}`}>
              {engine.totalStaked} ETH
            </span>
          </div>
        </div>
        
        <div className={`text-xs matrix-text ${getStatusColor(engine.status || "offline")}`} data-testid={`status-${engine.id}`}>
          {(engine.status || "offline").toUpperCase()} • {engine.type === "human" ? "ACTIVE" : "READY"}
        </div>
      </CardContent>
    </Card>
  );
}
