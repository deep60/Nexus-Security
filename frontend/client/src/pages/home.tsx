import { Navigation } from "@/components/navigation";
import { ParticleBackground } from "@/components/particle-background";
import { ThreatVisualization } from "@/components/threat-visualization";
import { FileSubmissionForm } from "@/components/file-submission-form";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Link } from "wouter";
import { Upload, Code, Twitter, Github, MessageCircle, Brain } from "lucide-react";
import { useQuery } from "@tanstack/react-query";

export default function Home() {
  const { data: stats } = useQuery<any>({
    queryKey: ["/api/stats"],
  });

  return (
    <div className="min-h-screen bg-background text-foreground">
      <ParticleBackground />
      <Navigation />

      {/* Hero Section */}
      <section className="relative z-10 py-20 overflow-hidden">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <div className="space-y-8">
              <div className="space-y-4">
                <h1 className="text-5xl lg:text-6xl font-bold">
                  Decentralized
                  <span className="bg-gradient-to-r from-primary via-secondary to-accent bg-clip-text text-transparent">
                    {" "}Threat Intelligence{" "}
                  </span>
                  Marketplace
                </h1>
                <p className="text-xl text-muted-foreground leading-relaxed">
                  Crowdsource cybersecurity expertise to detect malware and cyber threats. 
                  Multi-engine analysis powered by blockchain incentives.
                </p>
              </div>
              
              <div className="flex flex-wrap gap-4">
                <Link href="/dashboard">
                  <Button className="glow-effect" size="lg" data-testid="button-submit-file">
                    <Upload className="w-5 h-5 mr-2" />
                    Submit File for Analysis
                  </Button>
                </Link>
                <Link href="/api">
                  <Button variant="outline" size="lg" data-testid="button-explore-api">
                    <Code className="w-5 h-5 mr-2" />
                    Explore API
                  </Button>
                </Link>
              </div>

              <div className="grid grid-cols-3 gap-6 pt-8">
                <div className="text-center">
                  <div className="text-3xl font-bold text-primary" data-testid="stat-files-analyzed">
                    {stats?.totalSubmissions || 0}
                  </div>
                  <div className="text-sm text-muted-foreground">Files Analyzed</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-accent" data-testid="stat-security-engines">
                    {stats?.totalEngines || 0}+
                  </div>
                  <div className="text-sm text-muted-foreground">Security Engines</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-secondary">99.2%</div>
                  <div className="text-sm text-muted-foreground">Accuracy Rate</div>
                </div>
              </div>
            </div>

            <div className="relative">
              <ThreatVisualization />
            </div>
          </div>
        </div>
      </section>

      {/* File Submission Section */}
      <section className="relative z-10 py-16 bg-muted/20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold mb-4">Submit for Analysis</h2>
            <p className="text-xl text-muted-foreground">Upload suspicious files or URLs and set bounties for expert analysis</p>
          </div>

          <div className="max-w-2xl mx-auto">
            <FileSubmissionForm />
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="relative z-10 py-16">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold mb-4">How It Works</h2>
            <p className="text-xl text-muted-foreground">Decentralized threat detection in four simple steps</p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
            <Card className="glassmorphism border-primary/20 text-center">
              <CardContent className="p-6">
                <div className="w-16 h-16 bg-primary/20 rounded-full flex items-center justify-center mx-auto mb-4">
                  <Upload className="w-8 h-8 text-primary" />
                </div>
                <h3 className="text-lg font-semibold mb-2">1. Submit</h3>
                <p className="text-sm text-muted-foreground">
                  Upload suspicious files or URLs with cryptocurrency bounties
                </p>
              </CardContent>
            </Card>

            <Card className="glassmorphism border-accent/20 text-center">
              <CardContent className="p-6">
                <div className="w-16 h-16 bg-accent/20 rounded-full flex items-center justify-center mx-auto mb-4">
                  <Brain className="w-8 h-8 text-accent" />
                </div>
                <h3 className="text-lg font-semibold mb-2">2. Analyze</h3>
                <p className="text-sm text-muted-foreground">
                  Multiple AI and human experts analyze and stake on results
                </p>
              </CardContent>
            </Card>

            <Card className="glassmorphism border-secondary/20 text-center">
              <CardContent className="p-6">
                <div className="w-16 h-16 bg-secondary/20 rounded-full flex items-center justify-center mx-auto mb-4">
                  <MessageCircle className="w-8 h-8 text-secondary" />
                </div>
                <h3 className="text-lg font-semibold mb-2">3. Consensus</h3>
                <p className="text-sm text-muted-foreground">
                  AI combines all analysis results into a confidence score
                </p>
              </CardContent>
            </Card>

            <Card className="glassmorphism border-primary/20 text-center">
              <CardContent className="p-6">
                <div className="w-16 h-16 bg-primary/20 rounded-full flex items-center justify-center mx-auto mb-4">
                  <Code className="w-8 h-8 text-primary" />
                </div>
                <h3 className="text-lg font-semibold mb-2">4. Integrate</h3>
                <p className="text-sm text-muted-foreground">
                  Access results via API or real-time WebSocket feeds
                </p>
              </CardContent>
            </Card>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="relative z-10 py-16 border-t border-border bg-muted/10">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid md:grid-cols-4 gap-8">
            <div className="space-y-4">
              <div className="text-2xl font-bold bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent">
                NEXUS-SECURITY
              </div>
              <p className="text-muted-foreground">
                Decentralized threat intelligence marketplace powered by blockchain technology.
              </p>
              <div className="flex space-x-4">
                <a 
                  href="#" 
                  className="w-10 h-10 bg-primary/20 rounded-lg flex items-center justify-center text-primary hover:bg-primary/30 transition-colors"
                  data-testid="link-twitter"
                >
                  <Twitter className="w-5 h-5" />
                </a>
                <a 
                  href="#" 
                  className="w-10 h-10 bg-primary/20 rounded-lg flex items-center justify-center text-primary hover:bg-primary/30 transition-colors"
                  data-testid="link-github"
                >
                  <Github className="w-5 h-5" />
                </a>
                <a 
                  href="#" 
                  className="w-10 h-10 bg-primary/20 rounded-lg flex items-center justify-center text-primary hover:bg-primary/30 transition-colors"
                  data-testid="link-discord"
                >
                  <MessageCircle className="w-5 h-5" />
                </a>
              </div>
            </div>

            <div>
              <h4 className="font-semibold mb-4">Platform</h4>
              <ul className="space-y-2 text-muted-foreground">
                <li><Link href="/dashboard"><a className="hover:text-primary transition-colors">Submit Analysis</a></Link></li>
                <li><Link href="/marketplace"><a className="hover:text-primary transition-colors">Browse Bounties</a></Link></li>
                <li><a href="#" className="hover:text-primary transition-colors">Engine Marketplace</a></li>
                <li><a href="#" className="hover:text-primary transition-colors">Reputation System</a></li>
              </ul>
            </div>

            <div>
              <h4 className="font-semibold mb-4">Developers</h4>
              <ul className="space-y-2 text-muted-foreground">
                <li><Link href="/api"><a className="hover:text-primary transition-colors">API Documentation</a></Link></li>
                <li><a href="#" className="hover:text-primary transition-colors">SDK Downloads</a></li>
                <li><a href="#" className="hover:text-primary transition-colors">Integration Guide</a></li>
                <li><a href="#" className="hover:text-primary transition-colors">Sample Code</a></li>
              </ul>
            </div>

            <div>
              <h4 className="font-semibold mb-4">Community</h4>
              <ul className="space-y-2 text-muted-foreground">
                <li><a href="#" className="hover:text-primary transition-colors">Discord Server</a></li>
                <li><a href="#" className="hover:text-primary transition-colors">Blog & Updates</a></li>
                <li><a href="#" className="hover:text-primary transition-colors">Bug Bounty Program</a></li>
                <li><a href="#" className="hover:text-primary transition-colors">Research Papers</a></li>
              </ul>
            </div>
          </div>

          <div className="border-t border-border mt-12 pt-8">
            <div className="flex flex-col md:flex-row justify-between items-center">
              <p className="text-muted-foreground text-sm">
                © 2024 Nexus-Security. All rights reserved.
              </p>
              <div className="flex space-x-6 text-sm text-muted-foreground mt-4 md:mt-0">
                <a href="#" className="hover:text-primary transition-colors">Privacy Policy</a>
                <a href="#" className="hover:text-primary transition-colors">Terms of Service</a>
                <a href="#" className="hover:text-primary transition-colors">Security</a>
              </div>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}
