import { Navigation } from "@/components/navigation";
import { ParticleBackground } from "@/components/particle-background";
import { EngineCard } from "@/components/engine-card";
import { BountyCard } from "@/components/bounty-card";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { useQuery } from "@tanstack/react-query";
import { PlusCircle, Info } from "lucide-react";

export default function Marketplace() {
  const { data: engines = [] } = useQuery<any[]>({
    queryKey: ["/api/engines"],
  });

  const { data: submissions = [] } = useQuery<any[]>({
    queryKey: ["/api/submissions"],
  });

  const { data: bounties = [] } = useQuery<any[]>({
    queryKey: ["/api/bounties"],
  });

  const { data: stats } = useQuery<any>({
    queryKey: ["/api/stats"],
  });

  // Combine submissions with bounties for display
  const activeBountySubmissions = submissions
    .filter((sub: any) => sub.status === "pending")
    .map((sub: any) => ({
      submission: sub,
      bounty: { amount: sub.bountyAmount, status: "active" },
    }));

  return (
    <div className="min-h-screen bg-background text-foreground">
      <ParticleBackground />
      <Navigation />

      <div className="relative z-10 py-8">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          {/* Security Engines Section */}
          <section className="mb-16">
            <div className="text-center mb-12">
              <h1 className="text-4xl font-bold mb-4">Security Engines</h1>
              <p className="text-xl text-muted-foreground">Compete and stake on analysis results</p>
            </div>

            <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6 mb-12">
              {engines.map((engine: any) => (
                <EngineCard key={engine.id} engine={engine} />
              ))}
            </div>

            {/* Become an Engine CTA */}
            <div className="text-center">
              <Card className="glassmorphism neon-border max-w-2xl mx-auto">
                <CardContent className="p-8">
                  <h3 className="text-2xl font-bold mb-4">Become a Security Engine</h3>
                  <p className="text-muted-foreground mb-6">
                    Join our network of security experts and automated tools. Stake cryptocurrency on your analysis accuracy and earn rewards for correct verdicts.
                  </p>
                  <div className="flex flex-wrap justify-center gap-4">
                    <Button className="glow-effect" data-testid="button-register-engine">
                      <PlusCircle className="w-4 h-4 mr-2" />
                      Register Engine
                    </Button>
                    <Button variant="outline" data-testid="button-learn-more">
                      <Info className="w-4 h-4 mr-2" />
                      Learn More
                    </Button>
                  </div>
                </CardContent>
              </Card>
            </div>
          </section>

          {/* Active Bounties Section */}
          <section className="mb-16">
            <div className="text-center mb-12">
              <h2 className="text-4xl font-bold mb-4">Active Bounties</h2>
              <p className="text-xl text-muted-foreground">Analyze files and earn cryptocurrency rewards</p>
            </div>

            {activeBountySubmissions.length === 0 ? (
              <Card className="glassmorphism text-center">
                <CardContent className="p-12">
                  <p className="text-muted-foreground text-lg" data-testid="text-no-bounties">
                    No active bounties at the moment. Submit a file to create one!
                  </p>
                </CardContent>
              </Card>
            ) : (
              <div className="grid lg:grid-cols-2 gap-8 mb-12">
                {activeBountySubmissions.slice(0, 4).map(({ submission, bounty }: any) => (
                  <BountyCard key={submission.id} submission={submission} bounty={bounty} />
                ))}
              </div>
            )}

            {/* Bounty Statistics */}
            <div className="grid md:grid-cols-4 gap-6">
              <Card className="glassmorphism text-center">
                <CardContent className="p-6">
                  <div className="text-3xl font-bold text-primary mb-2" data-testid="stat-total-bounties">
                    {stats?.totalActiveBounties || "0"} ETH
                  </div>
                  <div className="text-sm text-muted-foreground">Total Active Bounties</div>
                </CardContent>
              </Card>
              <Card className="glassmorphism text-center">
                <CardContent className="p-6">
                  <div className="text-3xl font-bold text-accent mb-2" data-testid="stat-files-queue">
                    {activeBountySubmissions.length}
                  </div>
                  <div className="text-sm text-muted-foreground">Files in Queue</div>
                </CardContent>
              </Card>
              <Card className="glassmorphism text-center">
                <CardContent className="p-6">
                  <div className="text-3xl font-bold text-secondary mb-2">
                    {stats?.avgResponseTime || "24.7s"}
                  </div>
                  <div className="text-sm text-muted-foreground">Avg Response Time</div>
                </CardContent>
              </Card>
              <Card className="glassmorphism text-center">
                <CardContent className="p-6">
                  <div className="text-3xl font-bold text-primary mb-2">
                    {stats?.totalRewardsPaid || "312.8"} ETH
                  </div>
                  <div className="text-sm text-muted-foreground">Rewards Paid Out</div>
                </CardContent>
              </Card>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
