import { Navigation } from "@/components/navigation";
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
      code: `docker run -v ./rules:/rules nexus/worker`
    },
    {
      icon: Network,
      title: "Decentralized Architecture",
      description: "Nodes compute entirely independently, preventing targeted attacks against a monolithic infrastructure.",
      code: `Node ID: nd_01h...\nStatus: Connected`
    }
  ];

  return (
    <div className="min-h-screen bg-slate-950 text-slate-300 font-sans selection:bg-blue-500/30">
      <SEO 
        title="Features & Capabilities" 
        description="Technical capabilities of the Nexus-Security engine. API-first, verifiable, multi-engine threat detection."
      />
      <Navigation />
      
      <main className="pt-32 pb-24">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-20 max-w-3xl mx-auto">
            <h1 className="text-4xl md:text-5xl font-bold font-sans text-white tracking-tight mb-6">
              Technical Capabilities
            </h1>
            <p className="text-lg md:text-xl text-slate-400 leading-relaxed font-sans">
              Built for speed, accuracy, and transparency. Designed to integrate seamlessly into modern security operations.
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            {features.map((feature, idx) => (
              <div key={idx} className="group relative bg-slate-900/30 border border-slate-800/60 rounded-xl overflow-hidden hover:border-blue-500/50 transition-colors">
                <div className="p-8">
                  <feature.icon className="w-8 h-8 text-blue-400 mb-6 group-hover:scale-110 transition-transform" />
                  <h3 className="text-xl font-bold text-white mb-3 font-sans">{feature.title}</h3>
                  <p className="text-slate-400 text-sm leading-relaxed mb-6">
                    {feature.description}
                  </p>
                </div>
                <div className="bg-slate-950 border-t border-slate-800/60 p-4">
                  <pre className="text-xs font-mono text-emerald-400/80 overflow-x-auto whitespace-pre-wrap">
                    <code>{feature.code}</code>
                  </pre>
                </div>
              </div>
            ))}
          </div>
        </div>
      </main>

      <footer className="border-t border-slate-800 py-12 bg-slate-950 mt-12">
        <div className="max-w-6xl mx-auto px-4 flex flex-col items-center">
          <div className="text-slate-500 font-mono text-sm mb-4 bg-slate-900 px-4 py-1 rounded-full border border-slate-800">nexus-security // features</div>
          <p className="text-slate-600 text-xs">© 2026 Nexus-Security. Developer First.</p>
        </div>
      </footer>
    </div>
  );
}
