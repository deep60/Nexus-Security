import { Navigation } from "@/components/navigation";
import { Footer } from "@/components/footer";
import { Terminal, Shield, Zap, Cpu, Database, Network } from "lucide-react";
import { SEO } from "@/components/seo";

export default function Features() {
  const features = [
    {
      icon: Terminal,
      title: "API-First Architecture",
      description: "Everything you can do in the dashboard, you can do via the API. Built with gRPC and REST endpoints.",
      code: `POST /v1/analysis\n{ "hash": "e3b0c442...",\n  "engines": ["yara"] }`
    },
    {
      icon: Shield,
      title: "Verifiable Consensus",
      description: "Results are cryptographically signed. Stop guessing why an AV flagged your file.",
      code: `"consensus": {\n  "score": 98.5,\n  "signatures": ["0x..."]\n}`
    },
    {
      icon: Cpu,
      title: "Multi-Engine Support",
      description: "YARA, ClamAV, and custom ML engines (ONNX) run in parallel to eliminate single-engine bias.",
      code: `engines: ["yara", "clamav", "ml-onnx"]`
    },
    {
      icon: Zap,
      title: "Real-time Webhooks",
      description: "Get notified the millisecond an analysis completes. Ideal for blocking CI/CD pipelines instantly.",
      code: `{\n  "event": "analysis.complete",\n  "status": "malicious"\n}`
    },
    {
      icon: Database,
      title: "Private Rule Processing",
      description: "Deploy a local node to scan payloads against your proprietary YARA rules without uploading the rules.",
      code: `docker run -v ./rules:/rules verdyx/worker`
    },
    {
      icon: Network,
      title: "Decentralized Architecture",
      description: "Nodes compute entirely independently, preventing targeted attacks against a monolithic infrastructure.",
      code: `Node ID: nd_01h...\nStatus: Connected`
    }
  ];

  return (
    <div className="min-h-screen bg-background text-muted-foreground font-sans selection:bg-primary/30">
      <SEO 
        title="Features & Capabilities" 
        description="Technical capabilities of the Verdyx engine. API-first, verifiable, multi-engine threat detection."
      />
      <Navigation />
      
      <main className="pt-32 pb-24">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-20 max-w-3xl mx-auto">
            <h1 className="text-4xl md:text-5xl font-bold font-display text-foreground tracking-tight mb-6">
              Technical Capabilities
            </h1>
            <p className="text-lg md:text-xl text-muted-foreground leading-relaxed">
              Built for speed, accuracy, and transparency. Designed to integrate seamlessly into modern security operations.
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            {features.map((feature, idx) => (
              <div key={idx} className="group relative bg-card/40 border border-border/60 rounded-xl overflow-hidden hover:border-primary/50 transition-colors">
                <div className="p-8">
                  <feature.icon className="w-8 h-8 text-primary mb-6 group-hover:scale-110 transition-transform" />
                  <h3 className="text-xl font-bold text-foreground mb-3">{feature.title}</h3>
                  <p className="text-muted-foreground text-sm leading-relaxed mb-6">
                    {feature.description}
                  </p>
                </div>
                <div className="surface-panel border-x-0 border-b-0 border-t p-4">
                  <pre className="text-xs font-mono text-accent/90 overflow-x-auto whitespace-pre-wrap">
                    <code>{feature.code}</code>
                  </pre>
                </div>
              </div>
            ))}
          </div>
        </div>
      </main>

      <Footer tag="features" />
    </div>
  );
}
