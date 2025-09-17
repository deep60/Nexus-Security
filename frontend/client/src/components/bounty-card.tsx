import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { FileCode, Link as LinkIcon, Search } from "lucide-react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { apiRequest } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";

interface BountyCardProps {
  submission: any;
  bounty: any;
}

export function BountyCard({ submission, bounty }: BountyCardProps) {
  const { toast } = useToast();
  const queryClient = useQueryClient();

  const startAnalysisMutation = useMutation({
    mutationFn: async (submissionId: string) => {
      const response = await apiRequest("POST", `/api/submissions/${submissionId}/start-analysis`, {});
      return response.json();
    },
    onSuccess: () => {
      toast({
        title: "Analysis Started",
        description: "Your analysis has been submitted to the network",
      });
      queryClient.invalidateQueries({ queryKey: ["/api/submissions"] });
    },
    onError: () => {
      toast({
        title: "Failed to start analysis",
        description: "Please try again",
        variant: "destructive",
      });
    },
  });

  const handleAnalyze = () => {
    startAnalysisMutation.mutate(submission.id);
  };

  const getFileIcon = (submissionType: string) => {
    return submissionType === "url" ? <LinkIcon className="w-6 h-6" /> : <FileCode className="w-6 h-6" />;
  };

  const getBountyLevel = (amount: number) => {
    if (amount >= 2) return { level: "High", color: "border-secondary/20" };
    if (amount >= 0.5) return { level: "Medium", color: "border-primary/20" };
    return { level: "Low", color: "border-accent/20" };
  };

  const bountyAmount = parseFloat(bounty.amount);
  const bountyInfo = getBountyLevel(bountyAmount);

  return (
    <Card className={`glassmorphism ${bountyInfo.color} ${bountyAmount >= 2 ? 'glow-effect' : ''}`}>
      <CardContent className="p-6">
        <div className="flex justify-between items-start mb-4">
          <div className="flex items-center space-x-3">
            <div className={`w-14 h-14 ${
              submission.submissionType === "url" ? "bg-accent/20" : "bg-destructive/20"
            } rounded-lg flex items-center justify-center`}>
              {getFileIcon(submission.submissionType)}
            </div>
            <div>
              <div className="font-semibold text-lg" data-testid={`bounty-filename-${submission.id}`}>
                {submission.filename}
              </div>
              <div className="text-sm text-muted-foreground capitalize">
                {submission.analysisType} Analysis Required
              </div>
            </div>
          </div>
          <div className="text-right">
            <div className="text-2xl font-bold text-secondary" data-testid={`bounty-amount-${submission.id}`}>
              {bounty.amount} ETH
            </div>
            <div className="text-sm text-muted-foreground">Bounty</div>
          </div>
        </div>
        
        <div className="space-y-3 mb-6">
          <div className="flex justify-between text-sm">
            <span>File Size</span>
            <span className="font-mono">
              {submission.fileSize ? `${(submission.fileSize / 1024 / 1024).toFixed(1)} MB` : "N/A"}
            </span>
          </div>
          <div className="flex justify-between text-sm">
            <span>Submission Time</span>
            <span>{new Date(submission.createdAt).toLocaleString()}</span>
          </div>
          <div className="flex justify-between text-sm">
            <span>Priority</span>
            <span>
              {submission.priority ? (
                <Badge variant="outline" className="bg-secondary/20 text-secondary border-secondary/30">
                  High Priority
                </Badge>
              ) : (
                "Standard"
              )}
            </span>
          </div>
        </div>

        {submission.description && (
          <div className="bg-muted/30 rounded-lg p-4 mb-4">
            <div className="text-sm font-medium mb-2">Description:</div>
            <div className="text-sm text-muted-foreground" data-testid={`description-${submission.id}`}>
              {submission.description}
            </div>
          </div>
        )}

        <Button
          onClick={handleAnalyze}
          disabled={startAnalysisMutation.isPending || submission.status !== "pending"}
          className="w-full"
          variant={submission.status !== "pending" ? "secondary" : "default"}
          data-testid={`button-analyze-${submission.id}`}
        >
          <Search className="w-4 h-4 mr-2" />
          {submission.status === "pending" 
            ? (startAnalysisMutation.isPending ? "Starting..." : "Analyze & Stake")
            : submission.status === "analyzing" 
            ? "Analysis in Progress"
            : "Analysis Complete"
          }
        </Button>
      </CardContent>
    </Card>
  );
}