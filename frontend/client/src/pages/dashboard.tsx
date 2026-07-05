import { Navigation } from "@/components/navigation";
import { DashboardStats, RecentAnalyses } from "@/components/analysis-dashboard";
import { FileSubmissionForm } from "@/components/file-submission-form";
import { PlatformAnalytics } from "@/components/platform-analytics";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Activity, BarChart3, Upload } from "lucide-react";
import { SEO } from "@/components/seo";
import { useWebSocket } from "@/hooks/use-websocket";

export default function Dashboard() {
  const { isConnected } = useWebSocket();

  return (
    <div className="min-h-screen bg-background text-foreground">
      <SEO title="Dashboard" description="Submit files for analysis and view recent intelligence reports." />
      <Navigation />

      <main id="main-content" tabIndex={-1} className="py-8">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          {/* Header */}
          <div className="mb-8 flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <h1 className="text-3xl font-bold font-display tracking-tight">Analysis Dashboard</h1>
              <p className="mt-1 text-muted-foreground">
                Monitor real-time threat detection and platform analytics
              </p>
            </div>
            <div className="inline-flex items-center gap-2 self-start rounded-full border border-border bg-card/50 px-3 py-1.5 text-sm sm:self-auto">
              <span className={`relative flex h-2.5 w-2.5`}>
                <span
                  className={`absolute inline-flex h-full w-full rounded-full ${isConnected ? "animate-ping bg-accent/60" : ""}`}
                />
                <span className={`relative inline-flex h-2.5 w-2.5 rounded-full ${isConnected ? "bg-accent" : "bg-muted-foreground"}`} />
              </span>
              <span className="font-mono text-xs text-muted-foreground">
                {isConnected ? "Live · connected" : "Reconnecting…"}
              </span>
            </div>
          </div>

          <Tabs defaultValue="live" className="space-y-8">
            <TabsList className="grid w-full max-w-md grid-cols-2">
              <TabsTrigger value="live">
                <Activity className="mr-2 h-4 w-4" />
                Live Analysis
              </TabsTrigger>
              <TabsTrigger value="analytics">
                <BarChart3 className="mr-2 h-4 w-4" />
                Platform Analytics
              </TabsTrigger>
            </TabsList>

            <TabsContent value="live" className="space-y-8">
              {/* Uniform KPI row */}
              <DashboardStats />

              {/* Balanced workspace: results (wider) + submit (sidebar) */}
              <div className="grid grid-cols-1 gap-8 lg:grid-cols-3">
                <div className="lg:col-span-2">
                  <RecentAnalyses />
                </div>

                <div className="lg:col-span-1">
                  <Card className="glassmorphism h-full">
                    <CardHeader>
                      <CardTitle className="flex items-center gap-2 text-lg font-semibold">
                        <Upload className="h-5 w-5 text-primary" />
                        Quick Submit
                      </CardTitle>
                      <CardDescription>Send a file or URL for consensus analysis.</CardDescription>
                    </CardHeader>
                    <CardContent>
                      <FileSubmissionForm />
                    </CardContent>
                  </Card>
                </div>
              </div>
            </TabsContent>

            <TabsContent value="analytics" className="space-y-8">
              <PlatformAnalytics />
            </TabsContent>
          </Tabs>
        </div>
      </main>
    </div>
  );
}
