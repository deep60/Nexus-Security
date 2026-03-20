import { Navigation } from "@/components/navigation";
import { ArchitectureDiagram } from "@/components/diagrams/ArchitectureDiagram";
import { SEO } from "@/components/seo";

export default function HowItWorks() {
  return (
    <div className="min-h-screen bg-slate-950 text-slate-300 font-sans selection:bg-blue-500/30">
      <SEO 
        title="How It Works" 
        description="A transparent, verifiable pipeline replacing the legacy black-box heuristics of traditional antivirus."
      />
      <Navigation />
      
      <main className="pt-32 pb-24">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="mb-16">
            <h1 className="text-4xl md:text-5xl font-bold font-sans text-white tracking-tight mb-6">
              How Nexus-Security Works
            </h1>
            <p className="text-lg md:text-xl text-slate-400 leading-relaxed font-sans">
              A transparent, verifiable pipeline replacing the legacy black-box heuristics of traditional antivirus exactly.
            </p>
          </div>

          <div className="prose prose-invert prose-slate max-w-none">
            <div className="my-16">
              <ArchitectureDiagram />
            </div>

            <div className="space-y-16">
              <section className="border-t border-slate-800 pt-8">
                <h2 className="text-2xl font-bold text-white mb-4">1. Submission API Layer</h2>
                <p className="text-slate-400 mb-6">
                  Telemetry and files enter the system via high-throughput gRPC and REST endpoints. The gateway handles authentication, JWT validation securely, and payload deduplication securely. No file is executed at this layer.
                </p>
                <div className="bg-slate-900 border border-slate-800 rounded-md p-4 overflow-x-auto">
                  <pre className="text-sm font-mono text-emerald-400">
                    <code>
{`curl -X POST https://api.nexus-security.com/v1/analyze \\
  -H "Authorization: Bearer \$NEXUS_TOKEN" \\
  -F "file=@payload.bin" \\
  -F "engines=yara,clamav"`}
                    </code>
                  </pre>
                </div>
              </section>

              <section className="border-t border-slate-800 pt-8">
                <h2 className="text-2xl font-bold text-white mb-4">2. Distributed Analysis Nodes</h2>
                <p className="text-slate-400 mb-6">
                  The orchestrator dispatches the payload to multiple isolated worker nodes. Each node runs a specific engine tightly sandboxed. Customers can deploy private nodes to run proprietary YARA rules without exposing them to the public network.
                </p>
                <div className="bg-slate-900 border border-slate-800 rounded-md p-4 overflow-x-auto">
                  <pre className="text-sm font-mono text-blue-400">
                    <code>
{`# Docker Compose example for private node
services:
  nexus-worker:
    image: nexus/worker-yara:latest
    environment:
      - NEXUS_ORCHESTRATOR_URL=wss://api.nexus.com
      - PRIVATE_RULES_DIR=/rules
    volumes:
      - ./my-rules:/rules:ro`}
                    </code>
                  </pre>
                </div>
              </section>

              <section className="border-t border-slate-800 pt-8">
                <h2 className="text-2xl font-bold text-white mb-4">3. Consensus & Aggregation</h2>
                <p className="text-slate-400 mb-6">
                  Results from all nodes are collected asynchronously. The consensus engine applies a weighted algorithm to the signals. For example, a generic ClamAV hit might contribute 20% to the confidence score, while a highly specific YARA rule match contributes 80%. 
                </p>
                <p className="text-slate-400">
                  If the final confidence score exceeds the threshold, the payload is flagged. The entire cryptographic proof of the analysis is verifiable.
                </p>
              </section>
            </div>
          </div>
        </div>
      </main>

      <footer className="border-t border-slate-800 py-12 bg-slate-950 mt-20">
        <div className="max-w-6xl mx-auto px-4 flex flex-col items-center">
          <div className="text-slate-500 font-mono text-sm mb-4 bg-slate-900 px-4 py-1 rounded-full border border-slate-800">nexus-security // docs</div>
          <p className="text-slate-600 text-xs">© 2026 Nexus-Security. Developer First.</p>
        </div>
      </footer>
    </div>
  );
}
