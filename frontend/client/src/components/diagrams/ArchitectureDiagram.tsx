import { useState } from "react";
import { Card } from "@/components/ui/card";

export function ArchitectureDiagram() {
  const [activeLayer, setActiveLayer] = useState<string | null>(null);

  const layers = [
    {
      id: "api",
      title: "Submission API & Gateway",
      description: "gRPC & REST endpoints. Handles rate limiting, authentication, and file ingestion.",
      color: "text-blue-500",
      bgHover: "hover:bg-blue-500/10",
      borderHover: "hover:border-blue-500/50",
    },
    {
      id: "orchestrator",
      title: "Distributed Scanner Nodes",
      description: "Parallel execution across YARA, ClamAV, and ML engines (ONNX). Fully containerized and scalable.",
      color: "text-emerald-500",
      bgHover: "hover:bg-emerald-500/10",
      borderHover: "hover:border-emerald-500/50",
    },
    {
      id: "consensus",
      title: "Consensus Engine",
      description: "Aggregates results using a weighted algorithm to eliminate false positives. Optionally logs to blockchain.",
      color: "text-purple-500",
      bgHover: "hover:bg-purple-500/10",
      borderHover: "hover:border-purple-500/50",
    },
  ];

  return (
    <div className="flex flex-col md:flex-row gap-8 items-center justify-center p-6 w-full max-w-6xl mx-auto">
      {/* SVG Diagram Side */}
      <div className="w-full md:w-1/2 relative min-h-[400px] flex items-center justify-center">
        <svg
          viewBox="0 0 400 500"
          className="w-full max-w-sm h-auto drop-shadow-2xl"
          xmlns="http://www.w3.org/2000/svg"
        >
          <defs>
            <linearGradient id="cyberGradient" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="#3b82f6" stopOpacity="0.2" />
              <stop offset="100%" stopColor="#0ea5e9" stopOpacity="0.05" />
            </linearGradient>
            
            <linearGradient id="emeraldGradient" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="#10b981" stopOpacity="0.2" />
              <stop offset="100%" stopColor="#059669" stopOpacity="0.05" />
            </linearGradient>

            <linearGradient id="purpleGradient" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="#a855f7" stopOpacity="0.2" />
              <stop offset="100%" stopColor="#7e22ce" stopOpacity="0.05" />
            </linearGradient>
            
            <filter id="glow" x="-20%" y="-20%" width="140%" height="140%">
              <feGaussianBlur stdDeviation="5" result="blur" />
              <feComposite in="SourceGraphic" in2="blur" operator="over" />
            </filter>
          </defs>

          {/* Lines connecting layers */}
          <path d="M200 130 L200 180" stroke="#334155" strokeWidth="2" strokeDasharray="4 4" className="animate-pulse-slow" />
          <path d="M200 300 L200 350" stroke="#334155" strokeWidth="2" strokeDasharray="4 4" className="animate-pulse-slow" />

          {/* Layer 1: API */}
          <g 
            onMouseEnter={() => setActiveLayer("api")}
            onMouseLeave={() => setActiveLayer(null)}
            className="cursor-pointer transition-all duration-300 transform origin-center"
            style={{ transform: activeLayer === "api" ? "scale(1.02)" : "scale(1)" }}
          >
            <rect x="50" y="50" width="300" height="80" rx="8" fill="url(#cyberGradient)" stroke={activeLayer === "api" ? "#3b82f6" : "#1e293b"} strokeWidth="2" filter={activeLayer === "api" ? "url(#glow)" : ""} />
            <text x="200" y="90" textAnchor="middle" fill="#f8fafc" className="font-mono text-lg font-bold">1. Submission API</text>
            <text x="200" y="110" textAnchor="middle" fill="#94a3b8" className="font-sans text-xs">REST / gRPC Gateway</text>
          </g>

          {/* Layer 2: Scanners */}
          <g 
            onMouseEnter={() => setActiveLayer("orchestrator")}
            onMouseLeave={() => setActiveLayer(null)}
            className="cursor-pointer transition-all duration-300 transform origin-center"
            style={{ transform: activeLayer === "orchestrator" ? "scale(1.02)" : "scale(1)" }}
          >
            <rect x="30" y="180" width="340" height="120" rx="8" fill="url(#emeraldGradient)" stroke={activeLayer === "orchestrator" ? "#10b981" : "#1e293b"} strokeWidth="2" filter={activeLayer === "orchestrator" ? "url(#glow)" : ""} />
            <text x="200" y="215" textAnchor="middle" fill="#f8fafc" className="font-mono text-lg font-bold">2. Distributed Nodes</text>
            
            {/* Inner boxes for engines */}
            <rect x="50" y="235" width="80" height="40" rx="4" fill="#0f172a" stroke="#334155" />
            <text x="90" y="260" textAnchor="middle" fill="#10b981" className="font-mono text-xs">YARA</text>
            
            <rect x="160" y="235" width="80" height="40" rx="4" fill="#0f172a" stroke="#334155" />
            <text x="200" y="260" textAnchor="middle" fill="#10b981" className="font-mono text-xs">ClamAV</text>
            
            <rect x="270" y="235" width="80" height="40" rx="4" fill="#0f172a" stroke="#334155" />
            <text x="310" y="260" textAnchor="middle" fill="#10b981" className="font-mono text-xs">ML (ONNX)</text>
          </g>

          {/* Layer 3: Consensus */}
          <g 
            onMouseEnter={() => setActiveLayer("consensus")}
            onMouseLeave={() => setActiveLayer(null)}
            className="cursor-pointer transition-all duration-300 transform origin-center"
            style={{ transform: activeLayer === "consensus" ? "scale(1.02)" : "scale(1)" }}
          >
            <rect x="50" y="350" width="300" height="80" rx="8" fill="url(#purpleGradient)" stroke={activeLayer === "consensus" ? "#a855f7" : "#1e293b"} strokeWidth="2" filter={activeLayer === "consensus" ? "url(#glow)" : ""} />
            <text x="200" y="390" textAnchor="middle" fill="#f8fafc" className="font-mono text-lg font-bold">3. Consensus Engine</text>
            <text x="200" y="410" textAnchor="middle" fill="#94a3b8" className="font-sans text-xs">Aggregation & Verifiability</text>
          </g>
        </svg>
      </div>

      {/* Info Panel Side */}
      <div className="w-full md:w-1/2 flex flex-col gap-4">
        {layers.map((layer) => (
          <Card 
            key={layer.id}
            onMouseEnter={() => setActiveLayer(layer.id)}
            onMouseLeave={() => setActiveLayer(null)}
            className={`p-6 transition-all duration-300 border border-border/50 bg-card/50 backdrop-blur-sm cursor-pointer
              ${activeLayer === layer.id ? layer.bgHover + " " + layer.borderHover + " transform scale-[1.02]" : "hover:border-border/80"}
            `}
          >
            <h3 className={`font-mono text-xl font-bold mb-2 flex items-center gap-2 ${activeLayer === layer.id ? layer.color : "text-slate-200"}`}>
              <span className={`w-2 h-2 rounded-full ${activeLayer === layer.id ? "bg-current animate-pulse" : "bg-slate-600"}`} />
              {layer.title}
            </h3>
            <p className="text-slate-400 font-sans leading-relaxed">
              {layer.description}
            </p>
          </Card>
        ))}
      </div>
    </div>
  );
}
