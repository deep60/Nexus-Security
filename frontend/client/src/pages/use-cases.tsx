import { Navigation } from "@/components/navigation";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Activity, GitBranch, Search, ChevronRight } from "lucide-react";
import { SEO } from "@/components/seo";

export default function UseCases() {
  return (
    <div className="min-h-screen bg-slate-950 text-slate-300 font-sans selection:bg-blue-500/30">
      <SEO 
        title="Solutions by Role" 
        description="Built for SOC Teams, DevSecOps, and Threat Researchers. Nexus-Security adapts to your operational requirements."
      />
      <Navigation />
      
      <main className="pt-32 pb-24">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16 max-w-3xl mx-auto">
            <h1 className="text-4xl md:text-5xl font-bold font-sans text-white tracking-tight mb-6">
              Solutions by Role
            </h1>
            <p className="text-lg md:text-xl text-slate-400 leading-relaxed font-sans">
              Nexus-Security is built layer-by-layer to support the distinct operational requirements of modern security teams.
            </p>
          </div>

          <Tabs defaultValue="soc" className="w-full">
            <TabsList className="grid w-full grid-cols-1 md:grid-cols-3 bg-slate-900 border border-slate-800 p-1 mb-12 rounded-lg gap-2 h-auto">
              <TabsTrigger value="soc" className="py-3 px-6 data-[state=active]:bg-blue-600 data-[state=active]:text-white data-[state=inactive]:text-slate-400">
                <Activity className="w-4 h-4 mr-2" /> SOC Teams
              </TabsTrigger>
              <TabsTrigger value="devsecops" className="py-3 px-6 data-[state=active]:bg-emerald-600 data-[state=active]:text-white data-[state=inactive]:text-slate-400">
                <GitBranch className="w-4 h-4 mr-2" /> DevSecOps
              </TabsTrigger>
              <TabsTrigger value="research" className="py-3 px-6 data-[state=active]:bg-purple-600 data-[state=active]:text-white data-[state=inactive]:text-slate-400">
                <Search className="w-4 h-4 mr-2" /> Threat Researchers
              </TabsTrigger>
            </TabsList>

            <TabsContent value="soc" className="focus-visible:outline-none focus-visible:ring-0">
              <div className="grid md:grid-cols-2 gap-12 items-center">
                <div className="space-y-6">
                  <h2 className="text-3xl font-bold text-white">Automate Triage. Eliminate Alert Fatigue.</h2>
                  <p className="text-slate-400 text-lg">
                    Stop wasting analysts' time on sandbox setup. Feed suspicious payloads into Nexus-Security directly from your SOAR. The consensus engine filters out low-confidence noise.
                  </p>
                  <ul className="space-y-4">
                    {["SOAR platform integrations (Splunk, Cortex XSOAR).", "Verifiable audit logs for incident response reports.", "Instant bulk scanning via API without browser uploads."].map((item, i) => (
                      <li key={i} className="flex items-start text-slate-300">
                        <ChevronRight className="w-5 h-5 text-blue-500 mr-2 shrink-0 mt-0.5" />
                        <span>{item}</span>
                      </li>
                    ))}
                  </ul>
                </div>
                <div className="bg-slate-900/50 p-6 rounded-xl border border-slate-800">
                  <div className="flex items-center justify-between mb-4 border-b border-slate-800 pb-4">
                    <span className="font-mono text-sm text-slate-400">SOC Dashboard Feed</span>
                    <span className="flex h-3 w-3 relative">
                      <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-blue-400 opacity-75"></span>
                      <span className="relative inline-flex rounded-full h-3 w-3 bg-blue-500"></span>
                    </span>
                  </div>
                  <div className="space-y-4 font-mono text-xs">
                    <div className="bg-slate-950 p-3 rounded text-emerald-400 border border-slate-800">
                      [10:42:01] payload_a.exe ➔ Consensus: 12% (SAFE)
                    </div>
                    <div className="bg-slate-950 p-3 rounded text-red-400 border border-red-900/50 relative overflow-hidden">
                      <div className="absolute top-0 left-0 w-1 h-full bg-red-500"></div>
                      [10:42:05] payload_b.dll ➔ Consensus: 98% (MALICIOUS)
                      <br />
                      <span className="text-slate-500 mt-2 block">↳ Triggering EDR containment workflow...</span>
                    </div>
                  </div>
                </div>
              </div>
            </TabsContent>

            <TabsContent value="devsecops" className="focus-visible:outline-none focus-visible:ring-0">
              <div className="grid md:grid-cols-2 gap-12 items-center">
                <div className="space-y-6">
                  <h2 className="text-3xl font-bold text-white">CI/CD Pipeline Security at Scale.</h2>
                  <p className="text-slate-400 text-lg">
                    Don't let poisoned dependencies or injected malware reach production. Integrating Nexus-Security as a strict pre-flight check blocks deployments when consensus thresholds are breached.
                  </p>
                  <ul className="space-y-4">
                    {["Sub-second latency for artifact scanning.", "Containerized local nodes for air-gapped CI environments.", "GitHub Actions and GitLab CI ready."].map((item, i) => (
                      <li key={i} className="flex items-start text-slate-300">
                        <ChevronRight className="w-5 h-5 text-emerald-500 mr-2 shrink-0 mt-0.5" />
                        <span>{item}</span>
                      </li>
                    ))}
                  </ul>
                </div>
                <div className="bg-slate-950 p-6 rounded-xl border border-slate-800 shadow-[0_0_30px_rgba(16,185,129,0.05)]">
                  <div className="flex gap-2 mb-4 border-b border-slate-800 pb-4">
                    <div className="w-3 h-3 rounded-full bg-red-500/80"></div>
                    <div className="w-3 h-3 rounded-full bg-yellow-500/80"></div>
                    <div className="w-3 h-3 rounded-full bg-green-500/80"></div>
                  </div>
                  <pre className="text-xs font-mono text-slate-300 overflow-x-auto">
                    <code>
{`jobs:
  security_scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Nexus-Security Scan
        run: |
          nexus-cli scan ./build-artifact.tar.gz \\
            --threshold 80 \\
            --fail-on-malicious
        env:
          NEXUS_TOKEN: \${{ secrets.NEXUS_TOKEN }}`}
                    </code>
                  </pre>
                </div>
              </div>
            </TabsContent>

            <TabsContent value="research" className="focus-visible:outline-none focus-visible:ring-0">
              <div className="grid md:grid-cols-2 gap-12 items-center">
                <div className="space-y-6">
                  <h2 className="text-3xl font-bold text-white">Verifiable Cryptographic Proofs.</h2>
                  <p className="text-slate-400 text-lg">
                    Reverse engineers and researchers can download the exact engine signatures, memory dumps, and execution traces that led to the consensus score.
                  </p>
                  <ul className="space-y-4">
                    {["Access historical scanning data via API.", "Deploy custom, private YARA rules to sandboxed nodes.", "Analyze polymorphic malware effectively asynchronously."].map((item, i) => (
                      <li key={i} className="flex items-start text-slate-300">
                        <ChevronRight className="w-5 h-5 text-purple-500 mr-2 shrink-0 mt-0.5" />
                        <span>{item}</span>
                      </li>
                    ))}
                  </ul>
                </div>
                <div className="bg-slate-900/50 p-6 rounded-xl border border-slate-800">
                  <div className="space-y-3 font-mono text-sm text-slate-400">
                    <div className="text-purple-400 font-bold mb-4">Cryptographic receipt block #12093</div>
                    <div className="flex justify-between border-b border-slate-800 pb-2">
                       <span>Target Hash</span>
                       <span className="text-slate-200">a94a8fe5ccb19...</span>
                    </div>
                    <div className="flex justify-between border-b border-slate-800 pb-2">
                       <span>Engine 1 (YARA) Signature</span>
                       <span className="text-slate-200">0x4b2a...e91c</span>
                    </div>
                    <div className="flex justify-between">
                       <span>Engine 2 (ClamAV) Signature</span>
                       <span className="text-slate-200">0x7a11...f002</span>
                    </div>
                  </div>
                </div>
              </div>
            </TabsContent>
          </Tabs>
        </div>
      </main>

      <footer className="border-t border-slate-800 py-12 bg-slate-950 mt-12">
        <div className="max-w-6xl mx-auto px-4 flex flex-col items-center">
          <div className="text-slate-500 font-mono text-sm mb-4 bg-slate-900 px-4 py-1 rounded-full border border-slate-800">nexus-security // solutions</div>
          <p className="text-slate-600 text-xs">© 2026 Nexus-Security. Developer First.</p>
        </div>
      </footer>
    </div>
  );
}
