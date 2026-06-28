import { Navigation } from "@/components/navigation";
import { ArchitectureDiagram } from "@/components/diagrams/ArchitectureDiagram";
import { Footer } from "@/components/footer";
import { SEO } from "@/components/seo";

export default function HowItWorks() {
  return (
    <div className="min-h-screen bg-background text-muted-foreground font-sans selection:bg-primary/30">
      <SEO 
        title="How It Works" 
        description="A transparent, verifiable pipeline replacing the legacy black-box heuristics of traditional antivirus."
      />
      <Navigation />
      
      <main className="pt-32 pb-24">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="mb-16">
            <h1 className="text-4xl md:text-5xl font-bold font-display text-foreground tracking-tight mb-6">
              How Verdyx Works
            </h1>
            <p className="text-lg md:text-xl text-muted-foreground leading-relaxed">
              A transparent, verifiable pipeline replacing the legacy black-box heuristics of traditional antivirus.
            </p>
          </div>

          <div className="my-16">
            <ArchitectureDiagram />
          </div>

          <div className="space-y-16">
            <section className="border-t border-border pt-8">
              <h2 className="text-2xl font-bold text-foreground mb-4">1. Submission API Layer</h2>
              <p className="text-muted-foreground mb-6">
                Telemetry and files enter the system via high-throughput gRPC and REST endpoints. The gateway handles authentication, JWT validation, and payload deduplication. No file is executed at this layer.
              </p>
              <div className="surface-panel rounded-md p-4 overflow-x-auto">
                <pre className="text-sm font-mono text-accent">
                  <code>{`curl -X POST https://api.verdyx.com/v1/analyze \\ 
                  -H "Authorization: Bearer $VERDYX_TOKEN" \\
                  -F "file=@payload.bin" \\
                  -F "engines=yara,clamav"`}
                  </code>
                </pre>
              </div>
            </section>

            <section className="border-t border-border pt-8">
              <h2 className="text-2xl font-bold text-foreground mb-4">2. Distributed Analysis Nodes</h2>
              <p className="text-muted-foreground mb-6">
                The orchestrator dispatches the payload to multiple isolated worker nodes. Each node runs a specific engine in a tightly sandboxed environment. Customers can deploy private nodes to run proprietary YARA rules without exposing them to the public network.
              </p>
              <div className="surface-panel rounded-md p-4 overflow-x-auto">
                <pre className="text-sm font-mono text-primary">
                  <code>
{`# Docker Compose example for private node
services:
  verdyx-worker:
    image: verdyx/worker-yara:latest
    environment:
      - VERDYX_ORCHESTRATOR_URL=wss://api.verdyx.com
      - PRIVATE_RULES_DIR=/rules
    volumes:
      - ./my-rules:/rules:ro`}
                  </code>
                </pre>
              </div>
            </section>

            <section className="border-t border-border pt-8">
              <h2 className="text-2xl font-bold text-foreground mb-4">3. Consensus &amp; Aggregation</h2>
              <p className="text-muted-foreground mb-6">
                Results from all nodes are collected asynchronously. The consensus engine applies a weighted algorithm to the signals. For example, a generic ClamAV hit might contribute 20% to the confidence score, while a highly specific YARA rule match contributes 80%.
              </p>
              <p className="text-muted-foreground">
                If the final confidence score exceeds the threshold, the payload is flagged. The entire cryptographic proof of the analysis is verifiable.
              </p>
            </section>
          </div>
        </div>
      </main>

      <Footer tag="docs" />
    </div>
  );
}
