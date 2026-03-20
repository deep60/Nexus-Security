import { Check, X } from "lucide-react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";

const features = [
  {
    name: "Verification Method",
    legacy: "Single Source of Truth",
    sandbox: "Heuristic Estimation",
    nexus: "Decentralized Consensus",
    nexusHighlight: true,
  },
  {
    name: "API First Design",
    legacy: <X className="h-4 w-4 text-slate-500 mx-auto" />,
    sandbox: <span className="text-yellow-500">Limited/Rate-capped</span>,
    nexus: <Check className="h-5 w-5 text-emerald-500 mx-auto" />,
  },
  {
    name: "Multi-Engine Scanning",
    legacy: <X className="h-4 w-4 text-slate-500 mx-auto" />,
    sandbox: <Check className="h-4 w-4 text-emerald-500/50 mx-auto" />,
    nexus: <Check className="h-5 w-5 text-emerald-500 mx-auto" />,
  },
  {
    name: "Vendor Lock-in",
    legacy: <span className="text-red-400">High</span>,
    sandbox: <span className="text-yellow-500">Moderate</span>,
    nexus: <span className="text-emerald-500">None (Open Data)</span>,
  },
  {
    name: "False Positive Reduction",
    legacy: "Manual Whitelisting",
    sandbox: "Low (Solo Engine)",
    nexus: "Aggregated Confidence Scoring",
    nexusHighlight: true,
  },
];

export function ComparisonTable() {
  return (
    <div className="w-full max-w-5xl mx-auto rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm overflow-hidden mt-10">
      <div className="p-6 md:p-8 bg-slate-900/50 border-b border-border/50">
        <h2 className="text-2xl md:text-3xl font-sans font-bold text-white mb-2">
          Why Engineers Choose Nexus
        </h2>
        <p className="text-slate-400 font-sans">
          We built the engine we wanted to use. No black boxes, no arbitrary rate limits, just verifiable results.
        </p>
      </div>
      
      <div className="p-0 overflow-x-auto">
        <Table>
          <TableHeader>
            <TableRow className="hover:bg-transparent border-border/50">
              <TableHead className="w-[200px] sm:w-[300px] text-slate-300 font-sans text-base py-4 pl-6">
                Capability
              </TableHead>
              <TableHead className="text-center text-slate-400 font-mono text-sm py-4">
                Legacy Antivirus
              </TableHead>
              <TableHead className="text-center text-slate-400 font-mono text-sm py-4">
                Single-Vendor Sandbox
              </TableHead>
              <TableHead className="text-center bg-blue-500/5 font-sans font-bold text-blue-400 text-base py-4 pr-6">
                <div className="flex flex-col items-center justify-center gap-1">
                  Nexus-Security
                  <Badge variant="outline" className="border-blue-500/30 text-blue-400 bg-blue-500/10 text-[10px] uppercase font-mono mt-1">
                    Distributed
                  </Badge>
                </div>
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {features.map((feature, index) => (
              <TableRow 
                key={index} 
                className={`
                  border-border/30 hover:bg-slate-800/20 transition-colors
                  ${index === features.length - 1 ? 'border-0' : ''}
                `}
              >
                <TableCell className="font-medium text-slate-200 py-5 pl-6 font-sans">
                  {feature.name}
                </TableCell>
                <TableCell className="text-center text-slate-400 font-mono text-sm">
                  {feature.legacy}
                </TableCell>
                <TableCell className="text-center text-slate-400 font-mono text-sm">
                  {feature.sandbox}
                </TableCell>
                <TableCell className="text-center bg-blue-500/5 py-5 pr-6 font-mono text-sm">
                  <span className={feature.nexusHighlight ? "text-emerald-400 font-semibold" : ""}>
                    {feature.nexus}
                  </span>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}
