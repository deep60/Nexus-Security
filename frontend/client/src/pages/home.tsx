import { Navigation } from "@/components/navigation";
import { HeroCanvas } from "@/components/animations/HeroCanvas";
import { ArchitectureDiagram } from "@/components/diagrams/ArchitectureDiagram";
import { ComparisonTable } from "@/components/tables/ComparisonTable";
import { Button } from "@/components/ui/button";
import { Link } from "wouter";
import { Terminal, Shield, Cpu, Zap, Activity, Code } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { SEO } from "@/components/seo";

export default function Home() {
  const { data: stats } = useQuery<any>({
    queryKey: ["/api/stats"],
  });

  return (
    <div className="min-h-screen bg-slate-950 text-slate-300 font-sans selection:bg-blue-500/30">
      <SEO 
        title="Decentralized Threat Intelligence" 
        description="Verifiable malware analysis, powered by consensus. The first distributed threat intelligence engine utilizing YARA, ClamAV, and ML."
      />
      <Navigation />

      {/* Hero Section */}
      <section className="relative min-h-[90vh] flex items-center justify-center overflow-hidden">
        <HeroCanvas />
        <div className="absolute inset-0 bg-gradient-to-b from-transparent via-slate-950/80 to-slate-950 z-0" />
        
        <div className="relative z-10 max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 text-center pt-20">
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-blue-500/10 border border-blue-500/20 text-blue-400 font-mono text-xs md:text-sm mb-8">
            <Activity className="w-4 h-4 animate-pulse" />
            <span>Engine v2.0 Live &mdash; 99.2% Consensus Accuracy</span>
          </div>
          
          <h1 className="text-4xl md:text-6xl lg:text-7xl font-bold font-sans text-white tracking-tight mb-8 drop-shadow-lg leading-tight">
            Verifiable malware analysis,<br />
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-emerald-400">
              powered by consensus.
            </span>
          </h1>
          
          <p className="text-lg md:text-xl text-slate-400 max-w-3xl mx-auto mb-10 leading-relaxed font-sans">
            The first distributed threat intelligence engine. Submit telemetry via API, let independent nodes run YARA, ClamAV, and ML engines, and receive a cryptographically backed consensus score.
          </p>

          <div className="flex flex-col sm:flex-row gap-4 justify-center items-center">
            <Link href="/dashboard">
              <Button size="lg" className="bg-blue-600 hover:bg-blue-500 text-white font-mono h-14 px-8 rounded-none border border-blue-400/50 shadow-[0_0_15px_rgba(59,130,246,0.3)] hover:shadow-[0_0_25px_rgba(59,130,246,0.5)] transition-all">
                <Terminal className="w-5 h-5 mr-3" />
                Start Free Query
              </Button>
            </Link>
            <Link href="/api">
              <Button size="lg" variant="outline" className="h-14 px-8 font-mono rounded-none border-slate-700 bg-slate-900/50 hover:bg-slate-800 hover:text-white transition-all text-slate-300">
                <Code className="w-5 h-5 mr-3" />
                View API Docs
              </Button>
            </Link>
          </div>
          
          {/* Trust Metrics */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8 mt-24 border-t border-slate-800/50 pt-12">
            {[
              { label: "Files Analyzed", value: stats?.totalSubmissions ? stats.totalSubmissions.toLocaleString() : "2.4M+" },
              { label: "Active Nodes", value: stats?.totalEngines ? stats.totalEngines.toString() : "45" },
              { label: "Avg Latency", value: "112ms" },
              { label: "False Positives", value: "< 0.01%" }
            ].map((stat, i) => (
              <div key={i} className="flex flex-col gap-1 items-center">
                <div className="text-2xl md:text-3xl font-mono font-bold text-emerald-400">{stat.value}</div>
                <div className="text-xs font-sans text-slate-500 uppercase tracking-wider">{stat.label}</div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* How It Works (Diagram) Section */}
      <section className="py-24 relative bg-slate-900/30 border-y border-slate-800/50">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold font-sans text-white mb-4">Architecture Deep Dive</h2>
            <p className="text-slate-400 font-sans max-w-2xl mx-auto text-lg hover:text-slate-300 transition-colors">
              A transparent pipeline replacing the black box of legacy AVs. Hover over the layers to explore the components.
            </p>
          </div>
          <ArchitectureDiagram />
        </div>
      </section>

      {/* Features Grid */}
      <section className="py-24">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid md:grid-cols-3 gap-6">
            {[
              {
                icon: Shield,
                title: "No Single Point of Failure",
                desc: "Don't trust one vendor's heuristics. We aggregate scores from YARA, ClamAV, and custom ONNX models to eliminate blind spots."
              },
              {
                icon: Zap,
                title: "Built for CI/CD",
                desc: "Sub-second gRPC and REST APIs to fail builds automatically when malware is pushed. No tedious dashboard clicks required."
              },
              {
                icon: Cpu,
                title: "Bring Your Own Engine",
                desc: "Deploy a node in your own infrastructure to run proprietary YARA rules without sharing them with the network."
              }
            ].map((Feature, idx) => (
              <div key={idx} className="p-8 border border-slate-800/50 bg-slate-900/20 hover:bg-slate-800/40 transition-colors group">
                <Feature.icon className="w-8 h-8 text-blue-500 mb-6 group-hover:scale-110 transition-transform duration-300" />
                <h3 className="text-xl font-bold text-white mb-3 font-sans">{Feature.title}</h3>
                <p className="text-slate-400 text-sm leading-relaxed">{Feature.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Comparison Section */}
      <section className="py-24 relative">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
          <ComparisonTable />
        </div>
      </section>
      
      {/* Footer Minimal */}
      <footer className="border-t border-slate-800 py-12 bg-slate-950">
        <div className="max-w-6xl mx-auto px-4 flex flex-col items-center">
          <div className="text-slate-500 font-mono text-sm mb-4 bg-slate-900 px-4 py-1 rounded-full border border-slate-800">nexus-security // v2.0-stable</div>
          <p className="text-slate-600 text-xs">© 2026 Nexus-Security. Developer First.</p>
        </div>
      </footer>
    </div>
  );
}
