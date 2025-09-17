import { Navigation } from "@/components/navigation";
import { ParticleBackground } from "@/components/particle-background";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Key, Copy } from "lucide-react";
import { useToast } from "@/hooks/use-toast";

export default function ApiDocs() {
  const { toast } = useToast();

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      toast({
        title: "Copied to clipboard",
        description: "Code snippet copied successfully",
      });
    } catch (error) {
      toast({
        title: "Failed to copy",
        description: "Please copy manually",
        variant: "destructive",
      });
    }
  };

  const codeExample = `import requests

# Submit file for analysis
response = requests.post(
    'https://api.nexus-security.io/v1/submit',
    headers={
        'Authorization': 'Bearer YOUR_API_KEY',
        'Content-Type': 'multipart/form-data'
    },
    files={
        'file': open('suspicious.exe', 'rb')
    },
    data={
        'bounty': 0.1,
        'priority': true
    }
)

analysis_id = response.json()['id']

# Get results
results = requests.get(
    f'https://api.nexus-security.io/v1/analysis/{analysis_id}'
)

print(f"Threat score: {results.json()['confidence']}")`;

  const endpoints = [
    {
      method: "POST",
      path: "/api/v1/submit",
      description: "Submit file or URL for analysis",
      methodColor: "bg-accent text-accent-foreground",
    },
    {
      method: "GET", 
      path: "/api/v1/analysis/:id",
      description: "Get analysis results",
      methodColor: "bg-primary text-primary-foreground",
    },
    {
      method: "GET",
      path: "/api/v1/engines", 
      description: "List available security engines",
      methodColor: "bg-primary text-primary-foreground",
    },
    {
      method: "GET",
      path: "/api/v1/bounties",
      description: "Get active bounties", 
      methodColor: "bg-primary text-primary-foreground",
    },
  ];

  return (
    <div className="min-h-screen bg-background text-foreground">
      <ParticleBackground />
      <Navigation />

      <div className="relative z-10 py-8">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-12">
            <h1 className="text-4xl font-bold mb-4">Developer API</h1>
            <p className="text-xl text-muted-foreground">Integrate threat intelligence into your applications</p>
          </div>

          <div className="grid lg:grid-cols-2 gap-8">
            {/* API Endpoints */}
            <Card className="glassmorphism neon-border">
              <CardHeader>
                <CardTitle>API Endpoints</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                {endpoints.map((endpoint, index) => (
                  <Card key={index} className="bg-card border-border">
                    <CardContent className="p-4">
                      <div className="flex items-center justify-between mb-2">
                        <Badge className={`font-mono text-sm ${endpoint.methodColor}`}>
                          {endpoint.method}
                        </Badge>
                        <span className="text-sm text-muted-foreground font-mono">
                          {endpoint.path}
                        </span>
                      </div>
                      <div className="text-sm">{endpoint.description}</div>
                    </CardContent>
                  </Card>
                ))}
              </CardContent>
            </Card>

            {/* Code Example */}
            <Card className="glassmorphism neon-border">
              <CardHeader>
                <div className="flex items-center justify-between">
                  <CardTitle>Integration Example</CardTitle>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => copyToClipboard(codeExample)}
                    data-testid="button-copy-code"
                  >
                    <Copy className="w-4 h-4" />
                  </Button>
                </div>
              </CardHeader>
              <CardContent>
                <Card className="bg-card border-border">
                  <CardContent className="p-4">
                    <pre className="text-sm font-mono text-muted-foreground overflow-x-auto">
                      <code className="matrix-text whitespace-pre-wrap">
                        {codeExample}
                      </code>
                    </pre>
                  </CardContent>
                </Card>
                
                <div className="space-y-4 mt-6">
                  <div className="flex items-center justify-between text-sm">
                    <span>Rate Limit</span>
                    <Badge variant="outline" className="font-mono">1000 req/hour</Badge>
                  </div>
                  <div className="flex items-center justify-between text-sm">
                    <span>Response Time</span>
                    <Badge variant="outline" className="font-mono">{"< 500ms"}</Badge>
                  </div>
                  <div className="flex items-center justify-between text-sm">
                    <span>Uptime</span>
                    <Badge className="bg-accent text-accent-foreground font-mono">99.9%</Badge>
                  </div>
                </div>

                <Button className="w-full mt-6" data-testid="button-get-api-key">
                  <Key className="w-4 h-4 mr-2" />
                  Get API Key
                </Button>
              </CardContent>
            </Card>
          </div>

          {/* API Documentation Sections */}
          <div className="mt-16 space-y-8">
            <Card className="glassmorphism">
              <CardHeader>
                <CardTitle>Authentication</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-muted-foreground mb-4">
                  All API requests require authentication using API keys. Include your API key in the Authorization header:
                </p>
                <Card className="bg-muted/10 border-border">
                  <CardContent className="p-4">
                    <code className="text-sm font-mono text-accent">
                      Authorization: Bearer YOUR_API_KEY
                    </code>
                  </CardContent>
                </Card>
              </CardContent>
            </Card>

            <Card className="glassmorphism">
              <CardHeader>
                <CardTitle>Response Format</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-muted-foreground mb-4">
                  All API responses are returned in JSON format with the following structure:
                </p>
                <Card className="bg-muted/10 border-border">
                  <CardContent className="p-4">
                    <pre className="text-sm font-mono text-accent">
{`{
  "id": "uuid",
  "status": "analyzing|completed|failed",
  "confidence": 94.2,
  "verdict": "malicious|clean|suspicious",
  "engines": [
    {
      "name": "DeepScan AI",
      "verdict": "malicious",
      "confidence": 96.5
    }
  ]
}`}
                    </pre>
                  </CardContent>
                </Card>
              </CardContent>
            </Card>

            <Card className="glassmorphism">
              <CardHeader>
                <CardTitle>WebSocket Real-time Updates</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-muted-foreground mb-4">
                  Connect to our WebSocket endpoint for real-time analysis updates:
                </p>
                <Card className="bg-muted/10 border-border">
                  <CardContent className="p-4">
                    <code className="text-sm font-mono text-accent">
                      wss://api.nexus-security.io/ws
                    </code>
                  </CardContent>
                </Card>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </div>
  );
}
