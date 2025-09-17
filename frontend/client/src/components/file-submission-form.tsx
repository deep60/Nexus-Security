import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Checkbox } from "@/components/ui/checkbox";
import { Card, CardContent } from "@/components/ui/card";
import { CloudUpload, Rocket } from "lucide-react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { apiRequest } from "@/lib/queryClient";
import { useToast } from "@/hooks/use-toast";

export function FileSubmissionForm() {
  const [filename, setFilename] = useState("");
  const [analysisType, setAnalysisType] = useState("full");
  const [bountyAmount, setBountyAmount] = useState("0.1");
  const [description, setDescription] = useState("");
  const [priority, setPriority] = useState(false);
  const [fileSize, setFileSize] = useState<number>(0);

  const { toast } = useToast();
  const queryClient = useQueryClient();

  const submitMutation = useMutation({
    mutationFn: async (data: any) => {
      const response = await apiRequest("POST", "/api/submissions", data);
      return response.json();
    },
    onSuccess: (data) => {
      toast({
        title: "File submitted for analysis",
        description: `Analysis ID: ${data.id}`,
      });
      queryClient.invalidateQueries({ queryKey: ["/api/submissions"] });
      // Reset form
      setFilename("");
      setDescription("");
      setBountyAmount("0.1");
      setPriority(false);
      setFileSize(0);
    },
    onError: () => {
      toast({
        title: "Submission failed",
        description: "Please try again",
        variant: "destructive",
      });
    },
  });

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setFilename(file.name);
      setFileSize(file.size);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!filename) {
      toast({
        title: "No file selected",
        description: "Please select a file to analyze",
        variant: "destructive",
      });
      return;
    }

    submitMutation.mutate({
      filename,
      fileSize,
      submissionType: "file",
      analysisType,
      bountyAmount: priority ? (parseFloat(bountyAmount) + 0.05).toString() : bountyAmount,
      priority,
      description: description || null,
    });
  };

  return (
    <Card className="glassmorphism neon-border">
      <CardContent className="p-8">
        <form onSubmit={handleSubmit} className="space-y-6">
          <div className="text-center">
            <label htmlFor="file-upload" className="cursor-pointer" data-testid="label-file-upload">
              <div className="w-24 h-24 mx-auto bg-primary/10 rounded-full flex items-center justify-center mb-4 hover:bg-primary/20 transition-colors glow-effect">
                <CloudUpload className="w-8 h-8 text-primary" />
              </div>
              <p className="text-lg font-semibold">Drop files here or click to browse</p>
              <p className="text-sm text-muted-foreground">Supports: EXE, DLL, APK, URL, ZIP (Max 100MB)</p>
            </label>
            <input
              id="file-upload"
              type="file"
              className="hidden"
              onChange={handleFileChange}
              accept=".exe,.dll,.apk,.zip,.pdf,.doc,.docx"
              data-testid="input-file"
            />
            {filename && (
              <p className="text-sm text-primary mt-2" data-testid="text-filename">
                Selected: {filename}
              </p>
            )}
          </div>

          <div className="grid md:grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium mb-2">Analysis Type</label>
              <Select value={analysisType} onValueChange={setAnalysisType}>
                <SelectTrigger data-testid="select-analysis-type">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="full">Full Analysis</SelectItem>
                  <SelectItem value="quick">Quick Scan</SelectItem>
                  <SelectItem value="deep">Deep Inspection</SelectItem>
                  <SelectItem value="behavioral">Behavioral Analysis</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">Bounty Amount (ETH)</label>
              <Input
                type="number"
                step="0.001"
                placeholder="0.1"
                value={bountyAmount}
                onChange={(e) => setBountyAmount(e.target.value)}
                data-testid="input-bounty-amount"
              />
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium mb-2">Description (Optional)</label>
            <Textarea
              rows={3}
              placeholder="Describe where you found this file or why it's suspicious..."
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="resize-none"
              data-testid="textarea-description"
            />
          </div>

          <div className="flex items-center space-x-3">
            <Checkbox
              id="priority"
              checked={priority}
              onCheckedChange={(checked) => setPriority(checked as boolean)}
              data-testid="checkbox-priority"
            />
            <label htmlFor="priority" className="text-sm">
              Priority Analysis (+0.05 ETH)
            </label>
          </div>

          <Button
            type="submit"
            className="w-full glow-effect"
            disabled={submitMutation.isPending}
            data-testid="button-submit-analysis"
          >
            <Rocket className="w-4 h-4 mr-2" />
            {submitMutation.isPending ? "Submitting..." : "Submit for Analysis"}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}
