import { Brain, Bot, Shield, Lock, Zap } from "lucide-react";

export function ThreatVisualization() {
  return (
    <div className="relative w-full h-96 glassmorphism rounded-2xl p-8 hologram-effect">
      <div className="absolute inset-4 border border-primary/30 rounded-xl" />
      <div className="absolute inset-8 border border-accent/20 rounded-lg" />
      
      {/* Central Node */}
      <div className="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-20 h-20 bg-gradient-to-br from-primary to-secondary rounded-full glow-effect animate-pulse-slow flex items-center justify-center">
        <Brain className="w-8 h-8 text-white" />
      </div>

      {/* Orbiting Elements */}
      <div className="absolute top-1/4 left-1/4 w-12 h-12 bg-accent/80 rounded-full animate-float flex items-center justify-center">
        <Bot className="w-6 h-6 text-white" />
      </div>
      <div 
        className="absolute top-1/3 right-1/4 w-10 h-10 bg-destructive/80 rounded-full animate-float flex items-center justify-center" 
        style={{ animationDelay: "1s" }}
      >
        <Zap className="w-4 h-4 text-white" />
      </div>
      <div 
        className="absolute bottom-1/4 left-1/3 w-14 h-14 bg-secondary/80 rounded-full animate-float flex items-center justify-center" 
        style={{ animationDelay: "2s" }}
      >
        <Shield className="w-6 h-6 text-white" />
      </div>
      <div 
        className="absolute bottom-1/3 right-1/3 w-8 h-8 bg-primary/80 rounded-full animate-float flex items-center justify-center" 
        style={{ animationDelay: "3s" }}
      >
        <Lock className="w-4 h-4 text-white" />
      </div>

      {/* Connection Lines SVG */}
      <svg className="absolute inset-0 w-full h-full pointer-events-none">
        <defs>
          <linearGradient id="lineGradient" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="hsl(195, 100%, 50%)" stopOpacity="0.6" />
            <stop offset="100%" stopColor="hsl(142, 85%, 55%)" stopOpacity="0.2" />
          </linearGradient>
        </defs>
        <line x1="50%" y1="50%" x2="25%" y2="25%" stroke="url(#lineGradient)" strokeWidth="2" opacity="0.6"/>
        <line x1="50%" y1="50%" x2="75%" y2="33%" stroke="url(#lineGradient)" strokeWidth="2" opacity="0.6"/>
        <line x1="50%" y1="50%" x2="33%" y2="75%" stroke="url(#lineGradient)" strokeWidth="2" opacity="0.6"/>
        <line x1="50%" y1="50%" x2="67%" y2="67%" stroke="url(#lineGradient)" strokeWidth="2" opacity="0.6"/>
      </svg>
    </div>
  );
}
