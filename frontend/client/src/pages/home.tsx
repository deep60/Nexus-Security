import { Navigation } from "@/components/navigation";
import { HeroCanvas } from "@/components/animations/HeroCanvas";
import { ArchitectureDiagram } from "@/components/diagrams/ArchitectureDiagram";
import { ComparisonTable } from "@/components/tables/ComparisonTable";
import { Footer } from "@/components/footer";
import { Button } from "@/components/ui/button";
import { Link } from "wouter";
import { Terminal, Shield, Cpu, Zap, Activity, Code } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { SEO } from "@/components/seo";
import type { ApiStats } from "@/lib/api-types";

export default function Home() {
  const { data: stats } = useQuery<ApiStats>({
    queryKey: ["/api/analysis/stats"],
  });

  return (
    <div className="min-h-screen bg-background text-muted-foreground font-sans selection:bg-primary/30">
      <SEO 
        title="Decentralized Threat Intelligence" 
        description="Verifiable malware analysis, powered by consensus. The first distributed threat intelligence engine utilizing YARA, ClamAV, and ML."
      />
      <Navigation />

      {/* Hero Section */}
      <section className="relative min-h-[90vh] flex items-center justify-center overflow-hidden">
        <HeroCanvas />
        <div className="absolute inset-0 bg-gradient-to-b from-transparent via-background/80 to-background z-0" />
        
        <div className="relative z-10 max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 text-center pt-20">
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-primary/10 border border-primary/20 text-primary font-mono text-xs md:text-sm mb-8">
            <Activity className="w-4 h-4 animate-pulse" />
            <span>Engine v2.0 Live &mdash; 99.2% Consensus Accuracy</span>
          </div>
          
          <h1 className="text-4xl md:text-6xl lg:text-7xl font-bold font-display text-foreground tracking-tight mb-8 leading-tight">
            Verifiable malware analysis,<br />
            <span className="text-gradient-brand">
              powered by consensus.
            </span>
          </h1>
          
          <p className="text-lg md:text-xl text-muted-foreground max-w-3xl mx-auto mb-10 leading-relaxed">
            The first distributed threat intelligence engine. Submit telemetry via API, let independent nodes run YARA, ClamAV, and ML engines, and receive a cryptographically backed consensus score.
          </p>

          <div className="flex flex-col sm:flex-row gap-4 justify-center items-center">
            <Link href="/dashboard">
              <Button size="lg" className="font-mono h-14 px-8 glow-effect">
                <Terminal className="w-5 h-5 mr-3" />
                Start Free Query
              </Button>
            </Link>
            <Link href="/api">
              <Button size="lg" variant="outline" className="h-14 px-8 font-mono">
                <Code className="w-5 h-5 mr-3" />
                View API Docs
              </Button>
            </Link>
          </div>
          
          {/* Trust Metrics */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8 mt-24 border-t border-border/50 pt-12">
            {[
              { label: "Files Analyzed", value: stats?.totalSubmissions ? stats.totalSubmissions.toLocaleString() : "2.4M+" },
              { label: "Active Nodes", value: stats?.totalEngines ? stats.totalEngines.toString() : "45" },
              { label: "Avg Latency", value: "112ms" },
              { label: "False Positives", value: "< 0.01%" }
            ].map((stat, i) => (
              <div key={i} className="flex flex-col gap-1 items-center">
                <div className="text-2xl md:text-3xl font-mono font-bold text-accent">{stat.value}</div>
                <div className="text-xs text-muted-foreground/80 uppercase tracking-wider">{stat.label}</div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* How It Works (Diagram) Section */}
      <section className="py-24 relative bg-surface/30 border-y border-border/50">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold font-display text-foreground mb-4">Architecture Deep Dive</h2>
            <p className="text-muted-foreground max-w-2xl mx-auto text-lg">
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
              <div key={idx} className="p-8 rounded-xl border border-border/60 bg-card/40 hover:border-primary/40 hover:bg-card/70 transition-colors group">
                <Feature.icon className="w-8 h-8 text-primary mb-6 group-hover:scale-110 transition-transform duration-300" />
                <h3 className="text-xl font-bold text-foreground mb-3">{Feature.title}</h3>
                <p className="text-muted-foreground text-sm leading-relaxed">{Feature.desc}</p>
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
      
      <Footer tag="v2.0-stable" />
    </div>
  );
}
