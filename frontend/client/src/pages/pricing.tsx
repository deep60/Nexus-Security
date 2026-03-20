import { Navigation } from "@/components/navigation";
import { Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Link } from "wouter";
import { SEO } from "@/components/seo";

export default function Pricing() {
  return (
    <div className="min-h-screen bg-slate-950 text-slate-300 font-sans selection:bg-blue-500/30">
      <SEO 
        title="Transparent Pricing" 
        description="Pay-as-you-go API credits for developers and enterprise volume tiers for SOCs. Zero hidden fees."
      />
      <Navigation />
      
      <main className="pt-32 pb-24">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-20 max-w-3xl mx-auto">
            <h1 className="text-4xl md:text-5xl font-bold font-sans text-white tracking-tight mb-6 mt-8">
              Transparent API Pricing
            </h1>
            <p className="text-lg md:text-xl text-slate-400 leading-relaxed font-sans mt-4">
              Pay strictly for the compute you consume. No required sales calls, no artificial rate limits, and zero hidden fees.
            </p>
          </div>

          <div className="grid md:grid-cols-2 gap-8 max-w-4xl mx-auto">
            
            {/* Developer Tier - Pay As You Go */}
            <div className="bg-slate-900/40 border border-slate-800 rounded-2xl p-8 hover:border-blue-500/30 transition-colors relative h-full flex flex-col">
              <div className="mb-8">
                <div className="inline-flex px-3 py-1 rounded-full bg-blue-500/10 text-blue-400 font-mono text-sm font-bold mb-4">Pay As You Go</div>
                <h2 className="text-3xl font-bold text-white mb-2 font-sans">Developer</h2>
                <div className="flex items-baseline gap-1 mt-4">
                  <span className="text-4xl font-bold text-white">$0.005</span>
                  <span className="text-slate-400 font-mono text-sm">/ analysis</span>
                </div>
                <p className="text-slate-400 mt-4 text-sm leading-relaxed">
                  Perfect for independent researchers, hobbyists, or CI/CD pipelines at small scale. 
                </p>
              </div>
              
              <ul className="space-y-4 mb-8 flex-grow">
                {[
                  "1,000 free requests per month",
                  "Access to standard YARA & ClamAV engines",
                  "Community Discord support",
                  "Max payload size: 50MB",
                  "Standard REST API access"
                ].map((feature, i) => (
                  <li key={i} className="flex gap-3 items-start">
                    <Check className="w-5 h-5 text-blue-500 shrink-0" />
                    <span className="text-slate-300">{feature}</span>
                  </li>
                ))}
              </ul>

              <Link href="/register">
                <Button className="w-full bg-blue-600 hover:bg-blue-500 text-white font-mono h-12">
                  Start Building
                </Button>
              </Link>
            </div>

            {/* Enterprise Tier */}
            <div className="bg-slate-900 border border-emerald-500/30 rounded-2xl p-8 relative shadow-[0_0_40px_rgba(16,185,129,0.05)] h-full flex flex-col">
              <div className="absolute top-0 right-0 transform translate-x-2 -translate-y-2">
                 <span className="bg-emerald-500 text-slate-950 text-xs font-bold px-3 py-1 font-mono rounded-tl-lg rounded-br-lg shadow-lg">PRODUCTION READY</span>
              </div>
              
              <div className="mb-8">
                <h2 className="text-3xl font-bold text-white mb-2 font-sans">Enterprise</h2>
                <div className="flex items-baseline gap-1 mt-4">
                  <span className="text-4xl font-bold text-white">Custom Volume</span>
                </div>
                <p className="text-slate-400 mt-4 text-sm leading-relaxed">
                  For SOCs and massive CI footprints. Bring your own private nodes and engines behind the firewall.
                </p>
              </div>
              
              <ul className="space-y-4 mb-8 flex-grow">
                {[
                  "Bulk volumetric discounts (< $0.001/req)",
                  "Access to ML (ONNX) experimental engines",
                  "Deploy private scanning nodes (Bring Your Own Engine)",
                  "Sub-second gRPC stream access",
                  "SLA guarantees and direct engineer support",
                  "Unlimited payload sizes (up to local disk limit)"
                ].map((feature, i) => (
                  <li key={i} className="flex gap-3 items-start">
                    <Check className="w-5 h-5 text-emerald-500 shrink-0" />
                    <span className="text-slate-300">{feature}</span>
                  </li>
                ))}
              </ul>

              <Button variant="outline" className="w-full h-12 font-mono border-slate-700 bg-transparent hover:bg-slate-800 text-white hover:text-white">
                Contact Sales
              </Button>
            </div>

          </div>
        </div>
      </main>

      <footer className="border-t border-slate-800 py-12 bg-slate-950 mt-12">
        <div className="max-w-6xl mx-auto px-4 flex flex-col items-center">
          <div className="text-slate-500 font-mono text-sm mb-4 bg-slate-900 px-4 py-1 rounded-full border border-slate-800">nexus-security // pricing</div>
          <p className="text-slate-600 text-xs">© 2026 Nexus-Security. Developer First.</p>
        </div>
      </footer>
    </div>
  );
}
