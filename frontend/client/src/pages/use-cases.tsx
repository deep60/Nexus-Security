import { Navigation } from "@/components/navigation";
import { Footer } from "@/components/footer";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Activity, GitBranch, Search, ChevronRight } from "lucide-react";
import { SEO } from "@/components/seo";

export default function UseCases() {
  return (
    <div className="min-h-screen bg-background text-muted-foreground font-sans selection:bg-primary/30">
      <SEO 
        title="Solutions by Role" 
        description="Built for SOC Teams, DevSecOps, and Threat Researchers. Verdyx adapts to your operational requirements."
      />
      <Navigation />
      
      <main id="main-content" tabIndex={-1} className="pt-32 pb-24">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16 max-w-3xl mx-auto">
            <h1 className="text-4xl md:text-5xl font-bold font-display text-foreground tracking-tight mb-6">
              Solutions by Role
            </h1>
            <p className="text-lg md:text-xl text-muted-foreground leading-relaxed">
              Verdyx is built layer-by-layer to support the distinct operational requirements of modern security teams.
            </p>
          </div>

          <Tabs defaultValue="soc" className="w-full">
            <TabsList className="grid w-full grid-cols-1 md:grid-cols-3 bg-card border border-border p-1 mb-12 rounded-lg gap-2 h-auto">
              <TabsTrigger value="soc" className="py-3 px-6 data-[state=active]:bg-primary data-[state=active]:text-primary-foreground data-[state=inactive]:text-muted-foreground">
                <Activity className="w-4 h-4 mr-2" /> SOC Teams
              </TabsTrigger>
              <TabsTrigger value="devsecops" className="py-3 px-6 data-[state=active]:bg-accent data-[state=active]:text-accent-foreground data-[state=inactive]:text-muted-foreground">
                <GitBranch className="w-4 h-4 mr-2" /> DevSecOps
              </TabsTrigger>
              <TabsTrigger value="research" className="py-3 px-6 data-[state=active]:bg-[hsl(var(--chart-3))] data-[state=active]:text-primary-foreground data-[state=inactive]:text-muted-foreground">
                <Search className="w-4 h-4 mr-2" /> Threat Researchers
              </TabsTrigger>
            </TabsList>

            <TabsContent value="soc" className="focus-visible:outline-none focus-visible:ring-0">
              <div className="grid md:grid-cols-2 gap-12 items-center">
                <div className="space-y-6">
                  <h2 className="text-3xl font-bold text-foreground">Automate Triage. Eliminate Alert Fatigue.</h2>
                  <p className="text-muted-foreground text-lg">
                    Stop wasting analysts' time on sandbox setup. Feed suspicious payloads into Verdyx directly from your SOAR. The consensus engine filters out low-confidence noise.
                  </p>
                  <ul className="space-y-4">
                    {["SOAR platform integrations (Splunk, Cortex XSOAR).", "Verifiable audit logs for incident response reports.", "Instant bulk scanning via API without browser uploads."].map((item, i) => (
                      <li key={i} className="flex items-start text-foreground/90">
                        <ChevronRight className="w-5 h-5 text-primary mr-2 shrink-0 mt-0.5" />
                        <span>{item}</span>
                      </li>
                    ))}
                  </ul>
                </div>
                <div className="surface-panel p-6 rounded-xl">
                  <div className="flex items-center justify-between mb-4 border-b border-border pb-4">
                    <span className="font-mono text-sm text-muted-foreground">SOC Dashboard Feed</span>
                    <span className="flex h-3 w-3 relative">
                      <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary opacity-75"></span>
                      <span className="relative inline-flex rounded-full h-3 w-3 bg-primary"></span>
                    </span>
                  </div>
                  <div className="space-y-4 font-mono text-xs">
                    <div className="bg-background p-3 rounded text-accent border border-border">
                      [10:42:01] payload_a.exe ➔ Consensus: 12% (SAFE)
                    </div>
                    <div className="bg-background p-3 rounded text-destructive border border-destructive/40 relative overflow-hidden">
                      <div className="absolute top-0 left-0 w-1 h-full bg-destructive"></div>
                      [10:42:05] payload_b.dll ➔ Consensus: 98% (MALICIOUS)
                      <br />
                      <span className="text-muted-foreground mt-2 block">↳ Triggering EDR containment workflow...</span>
                    </div>
                  </div>
                </div>
              </div>
            </TabsContent>

            <TabsContent value="devsecops" className="focus-visible:outline-none focus-visible:ring-0">
              <div className="grid md:grid-cols-2 gap-12 items-center">
                <div className="space-y-6">
                  <h2 className="text-3xl font-bold text-foreground">CI/CD Pipeline Security at Scale.</h2>
                  <p className="text-muted-foreground text-lg">
                    Don't let poisoned dependencies or injected malware reach production. Integrating Verdyx as a strict pre-flight check blocks deployments when consensus thresholds are breached.
                  </p>
                  <ul className="space-y-4">
                    {["Sub-second latency for artifact scanning.", "Containerized local nodes for air-gapped CI environments.", "GitHub Actions and GitLab CI ready."].map((item, i) => (
                      <li key={i} className="flex items-start text-foreground/90">
                        <ChevronRight className="w-5 h-5 text-accent mr-2 shrink-0 mt-0.5" />
                        <span>{item}</span>
                      </li>
                    ))}
                  </ul>
                </div>
                <div className="surface-panel p-6 rounded-xl">
                  <div className="flex gap-2 mb-4 border-b border-border pb-4">
                    <div className="w-3 h-3 rounded-full bg-destructive/80"></div>
                    <div className="w-3 h-3 rounded-full bg-warning/80"></div>
                    <div className="w-3 h-3 rounded-full bg-accent/80"></div>
                  </div>
                  <pre className="text-xs font-mono text-foreground/90 overflow-x-auto">
                    <code>
{`jobs:
  security_scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Verdyx Scan
        run: |
          verdyx-cli scan ./build-artifact.tar.gz \\
            --threshold 80 \\
            --fail-on-malicious
        env:
          VERDYX_TOKEN: \${{ secrets.VERDYX_TOKEN }}`}
                    </code>
                  </pre>
                </div>
              </div>
            </TabsContent>

            <TabsContent value="research" className="focus-visible:outline-none focus-visible:ring-0">
              <div className="grid md:grid-cols-2 gap-12 items-center">
                <div className="space-y-6">
                  <h2 className="text-3xl font-bold text-foreground">Verifiable Cryptographic Proofs.</h2>
                  <p className="text-muted-foreground text-lg">
                    Reverse engineers and researchers can download the exact engine signatures, memory dumps, and execution traces that led to the consensus score.
                  </p>
                  <ul className="space-y-4">
                    {["Access historical scanning data via API.", "Deploy custom, private YARA rules to sandboxed nodes.", "Analyze polymorphic malware effectively and asynchronously."].map((item, i) => (
                      <li key={i} className="flex items-start text-foreground/90">
                        <ChevronRight className="w-5 h-5 text-[hsl(var(--chart-3))] mr-2 shrink-0 mt-0.5" />
                        <span>{item}</span>
                      </li>
                    ))}
                  </ul>
                </div>
                <div className="surface-panel p-6 rounded-xl">
                  <div className="space-y-3 font-mono text-sm text-muted-foreground">
                    <div className="text-[hsl(var(--chart-3))] font-bold mb-4">Cryptographic receipt block #12093</div>
                    <div className="flex justify-between border-b border-border pb-2">
                       <span>Target Hash</span>
                       <span className="text-foreground">a94a8fe5ccb19...</span>
                    </div>
                    <div className="flex justify-between border-b border-border pb-2">
                       <span>Engine 1 (YARA) Signature</span>
                       <span className="text-foreground">0x4b2a...e91c</span>
                    </div>
                    <div className="flex justify-between">
                       <span>Engine 2 (ClamAV) Signature</span>
                       <span className="text-foreground">0x7a11...f002</span>
                    </div>
                  </div>
                </div>
              </div>
            </TabsContent>
          </Tabs>
        </div>
      </main>

      <Footer tag="solutions" />
    </div>
  );
}
