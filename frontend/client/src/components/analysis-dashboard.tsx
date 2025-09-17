import { useQuery } from "@tanstack/react-query";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Activity, CheckCircle, AlertTriangle, RotateCw, FileText, Link as LinkIcon, Shield, Zap } from "lucide-react";
import { useWebSocket } from "@/hooks/use-websocket";
import { useEffect } from "react";

export function AnalysisDashboard() {
  const { data: submissions = [], refetch } = useQuery<any[]>({
    queryKey: ["/api/submissions"],
  });

  const { data: stats } = useQuery<any>({
    queryKey: ["/api/stats"],
  });

  useWebSocket((message) => {
    if (message.type === 'analysis_updated' || message.type === 'analysis_completed' || message.type === 'new_submission') {
      refetch();
    }
  });

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "analyzing":
        return <RotateCw className="w-4 h-4 text-primary animate-spin" />;
      case "completed":
        return <CheckCircle className="w-4 h-4 text-accent" />;
      case "pending":
        return <Activity className="w-4 h-4 text-secondary" />;
      default:
        return <AlertTriangle className="w-4 h-4 text-destructive" />;
    }
  };

  const getFileIcon = (submissionType: string) => {
    return submissionType === "url" ? <LinkIcon className="w-6 h-6" /> : <FileText className="w-6 h-6" />;
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case "analyzing":
        return "border-primary/20 bg-primary/5";
      case "completed":
        return "border-accent/20 bg-accent/5";
      case "pending":
        return "border-secondary/20 bg-secondary/5";
      default:
        return "border-destructive/20 bg-destructive/5";
    }
  };

  return (
    <div className="space-y-8">
      {/* Stats Grid */}
      <div className="grid lg:grid-cols-3 gap-6">
        <Card className="glassmorphism border-accent/20">
          <CardContent className="p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold text-accent">Active Analysis</h3>
              <div className="w-3 h-3 bg-accent rounded-full animate-pulse" />
            </div>
            <div className="text-2xl font-bold mb-2" data-testid="text-active-analyses">
              {stats?.activeAnalyses || 0}
            </div>
            <div className="text-sm text-muted-foreground">Files being analyzed</div>
            <div className="scanning-animation h-1 w-full mt-4 rounded-full" />
          </CardContent>
        </Card>

        <Card className="glassmorphism border-primary/20">
          <CardContent className="p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold text-primary">Completed Today</h3>
              <CheckCircle className="w-5 h-5 text-primary" />
            </div>
            <div className="text-2xl font-bold mb-2" data-testid="text-completed-today">
              {stats?.completedToday || 0}
            </div>
            <div className="text-sm text-muted-foreground">Analyses completed</div>
          </CardContent>
        </Card>

        <Card className="glassmorphism border-destructive/20">
          <CardContent className="p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold text-destructive">Threats Detected</h3>
              <AlertTriangle className="w-5 h-5 text-destructive" />
            </div>
            <div className="text-2xl font-bold mb-2" data-testid="text-threats-detected">
              {Math.floor(stats?.threatsDetected || 0)}
            </div>
            <div className="text-sm text-muted-foreground">Malicious files found</div>
          </CardContent>
        </Card>
      </div>

      {/* Recent Analysis Results */}
      <Card className="glassmorphism neon-border">
        <CardContent className="p-6">
          <h3 className="text-xl font-semibold mb-6">Recent Analysis Results</h3>
          <div className="space-y-4">
            {submissions.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground" data-testid="text-no-submissions">
                No submissions yet. Submit a file to get started!
              </div>
            ) : (
              submissions.slice(0, 5).map((submission: any) => (
                <div
                  key={submission.id}
                  className={`flex items-center justify-between p-4 bg-card rounded-lg border ${getStatusColor(submission.status)}`}
                  data-testid={`analysis-result-${submission.id}`}
                >
                  <div className="flex items-center space-x-4">
                    <div className="w-12 h-12 bg-primary/20 rounded-lg flex items-center justify-center">
                      {getFileIcon(submission.submissionType)}
                    </div>
                    <div>
                      <div className="font-semibold" data-testid={`filename-${submission.id}`}>
                        {submission.filename}
                      </div>
                      <div className="text-sm text-muted-foreground font-mono">
                        SHA256: {submission.fileHash.substring(0, 12)}...
                      </div>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="flex items-center space-x-2 mb-1">
                      {getStatusIcon(submission.status)}
                      <span className="text-sm font-medium capitalize" data-testid={`status-${submission.id}`}>
                        {submission.status}
                      </span>
                    </div>
                    <div className="text-xs text-muted-foreground">
                      {submission.status === "completed" ? "Analysis complete" : "In progress"}
                    </div>
                  </div>
                  <div className="flex space-x-2">
                    <Badge variant="outline" className="bg-primary/20 text-primary border-primary/30">
                      {submission.bountyAmount} ETH
                    </Badge>
                    {submission.priority && (
                      <Badge variant="outline" className="bg-secondary/20 text-secondary border-secondary/30">
                        Priority
                      </Badge>
                    )}
                  </div>
                </div>
              ))
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

