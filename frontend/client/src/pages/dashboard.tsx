import { Navigation } from "@/components/navigation";
import { ParticleBackground } from "@/components/particle-background";
import { AnalysisDashboard } from "@/components/analysis-dashboard";
import { FileSubmissionForm } from "@/components/file-submission-form";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function Dashboard() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <ParticleBackground />
      <Navigation />

      <div className="relative z-10 py-8">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="mb-8">
            <h1 className="text-4xl font-bold mb-4">Analysis Dashboard</h1>
            <p className="text-xl text-muted-foreground">
              Monitor real-time threat detection and submit files for analysis
            </p>
          </div>

          <div className="grid lg:grid-cols-3 gap-8">
            <div className="lg:col-span-2">
              <AnalysisDashboard />
            </div>
            
            <div className="space-y-6">
              <Card className="glassmorphism neon-border">
                <CardHeader>
                  <CardTitle className="text-lg font-semibold">Quick Submit</CardTitle>
                </CardHeader>
                <CardContent>
                  <FileSubmissionForm />
                </CardContent>
              </Card>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
