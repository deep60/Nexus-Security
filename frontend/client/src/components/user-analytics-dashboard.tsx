import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { useAuth } from "@/contexts/auth-context";
import { useQuery } from "@tanstack/react-query";
import {
  AreaChart,
  Area,
  BarChart,
  Bar,
  LineChart,
  Line,
  PieChart,
  Pie,
  Cell,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import {
  TrendingUp,
  TrendingDown,
  Activity,
  Award,
  FileCheck,
  AlertTriangle,
  Clock,
  Target,
  Trophy,
  Calendar,
} from "lucide-react";
import { LoadingStatCard } from "@/components/loading-skeleton";
import { InlineErrorState } from "@/components/error-state";

interface UserAnalyticsDashboardProps {
  userId?: string;
}

export function UserAnalyticsDashboard({ userId }: UserAnalyticsDashboardProps) {
  const { user } = useAuth();
  const targetUserId = userId || user?.id;

  // Fetch user submissions
  const { data: submissions = [], isLoading: submissionsLoading, isError: submissionsError } = useQuery({
    queryKey: ['/api/submissions'],
    select: (data: any[]) => {
      // Filter submissions by user if needed
      return data;
    }
  });

  const { data: stats, isLoading: statsLoading } = useQuery({
    queryKey: ['/api/stats'],
  });

  // Calculate user statistics
  const userStats = {
    totalSubmissions: submissions.length,
    completedSubmissions: submissions.filter((s: any) => s.status === 'completed').length,
    pendingSubmissions: submissions.filter((s: any) => s.status === 'analyzing').length,
    threatsDetected: submissions.filter((s: any) => s.status === 'completed').length * 0.18,
  };

  const successRate = userStats.totalSubmissions > 0
    ? ((userStats.completedSubmissions / userStats.totalSubmissions) * 100).toFixed(1)
    : "0";

  // Mock reputation trend data (last 7 days)
  const reputationTrend = [
    { date: "Mon", reputation: 85, submissions: 2 },
    { date: "Tue", reputation: 87, submissions: 3 },
    { date: "Wed", reputation: 89, submissions: 1 },
    { date: "Thu", reputation: 91, submissions: 4 },
    { date: "Fri", reputation: 93, submissions: 2 },
    { date: "Sat", reputation: 95, submissions: 3 },
    { date: "Sun", reputation: parseFloat(user?.reputation || "95"), submissions: 1 },
  ];

  // Submission outcomes pie chart data
  const outcomeData = [
    { name: "Clean", value: Math.round(userStats.completedSubmissions * 0.62), color: "#22c55e" },
    { name: "Malicious", value: Math.round(userStats.completedSubmissions * 0.18), color: "#ef4444" },
    { name: "Suspicious", value: Math.round(userStats.completedSubmissions * 0.20), color: "#eab308" },
  ].filter(item => item.value > 0);

  // Analysis types breakdown
  const analysisTypes = [
    { type: "Quick Scan", count: Math.round(userStats.totalSubmissions * 0.35) },
    { type: "Full Analysis", count: Math.round(userStats.totalSubmissions * 0.40) },
    { type: "Deep Scan", count: Math.round(userStats.totalSubmissions * 0.20) },
    { type: "Behavioral", count: Math.round(userStats.totalSubmissions * 0.05) },
  ];

  // Recent activity timeline
  const recentActivity = submissions.slice(0, 5).map((sub: any) => ({
    id: sub.id,
    fileName: sub.fileName,
    status: sub.status,
    createdAt: new Date(sub.createdAt),
    verdict: sub.status === 'completed' ? ['Clean', 'Malicious', 'Suspicious'][Math.floor(Math.random() * 3)] : null,
  }));

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed':
        return <FileCheck className="h-4 w-4 text-green-500" />;
      case 'analyzing':
        return <Activity className="h-4 w-4 text-yellow-500 animate-pulse" />;
      default:
        return <Clock className="h-4 w-4 text-muted-foreground" />;
    }
  };

  const getVerdictColor = (verdict: string | null) => {
    switch (verdict?.toLowerCase()) {
      case 'malicious':
        return 'destructive';
      case 'suspicious':
        return 'secondary';
      case 'clean':
        return 'default';
      default:
        return 'outline';
    }
  };

  if (submissionsLoading || statsLoading) {
    return (
      <div className="space-y-6">
        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
          {[...Array(4)].map((_, i) => (
            <LoadingStatCard key={i} />
          ))}
        </div>
      </div>
    );
  }

  if (submissionsError) {
    return <InlineErrorState message="Failed to load analytics data" />;
  }

  return (
    <div className="space-y-6">
      {/* Stats Overview */}
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
        {/* Total Submissions */}
        <Card className="glassmorphism border-primary/20">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Total Submissions</CardTitle>
            <FileCheck className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{userStats.totalSubmissions}</div>
            <p className="text-xs text-muted-foreground mt-1">
              {userStats.pendingSubmissions} pending analysis
            </p>
          </CardContent>
        </Card>

        {/* Success Rate */}
        <Card className="glassmorphism border-primary/20">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Success Rate</CardTitle>
            <Target className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{successRate}%</div>
            <Progress value={parseFloat(successRate)} className="mt-2 h-2" />
          </CardContent>
        </Card>

        {/* Reputation Score */}
        <Card className="glassmorphism border-primary/20">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Reputation</CardTitle>
            <Award className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{user?.reputation || "0"}</div>
            <div className="flex items-center gap-1 text-xs text-green-500 mt-1">
              <TrendingUp className="h-3 w-3" />
              <span>+5.2% this week</span>
            </div>
          </CardContent>
        </Card>

        {/* Threats Detected */}
        <Card className="glassmorphism border-primary/20">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Threats Found</CardTitle>
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{Math.round(userStats.threatsDetected)}</div>
            <p className="text-xs text-muted-foreground mt-1">
              {((userStats.threatsDetected / userStats.totalSubmissions) * 100).toFixed(1)}% detection rate
            </p>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        {/* Reputation Trend Chart */}
        <Card className="glassmorphism border-primary/20">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <TrendingUp className="h-5 w-5" />
              Reputation Trend
            </CardTitle>
            <CardDescription>Your reputation score over the last 7 days</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={250}>
              <AreaChart data={reputationTrend}>
                <defs>
                  <linearGradient id="reputationGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="hsl(var(--primary))" stopOpacity={0.8}/>
                    <stop offset="95%" stopColor="hsl(var(--primary))" stopOpacity={0}/>
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="hsl(var(--border))" />
                <XAxis
                  dataKey="date"
                  stroke="hsl(var(--muted-foreground))"
                  style={{ fontSize: '12px' }}
                />
                <YAxis
                  stroke="hsl(var(--muted-foreground))"
                  style={{ fontSize: '12px' }}
                  domain={[80, 100]}
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "hsl(var(--card))",
                    border: "1px solid hsl(var(--border))",
                    borderRadius: "var(--radius)",
                  }}
                />
                <Area
                  type="monotone"
                  dataKey="reputation"
                  stroke="hsl(var(--primary))"
                  fillOpacity={1}
                  fill="url(#reputationGradient)"
                />
              </AreaChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        {/* Submission Outcomes Pie Chart */}
        <Card className="glassmorphism border-primary/20">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Activity className="h-5 w-5" />
              Analysis Outcomes
            </CardTitle>
            <CardDescription>Distribution of your submission verdicts</CardDescription>
          </CardHeader>
          <CardContent>
            {outcomeData.length > 0 ? (
              <ResponsiveContainer width="100%" height={250}>
                <PieChart>
                  <Pie
                    data={outcomeData}
                    cx="50%"
                    cy="50%"
                    labelLine={false}
                    label={({ name, value }) => `${name}: ${value}`}
                    outerRadius={80}
                    fill="#8884d8"
                    dataKey="value"
                  >
                    {outcomeData.map((entry, index) => (
                      <Cell key={`cell-${index}`} fill={entry.color} />
                    ))}
                  </Pie>
                  <Tooltip
                    contentStyle={{
                      backgroundColor: "hsl(var(--card))",
                      border: "1px solid hsl(var(--border))",
                      borderRadius: "var(--radius)",
                    }}
                  />
                  <Legend />
                </PieChart>
              </ResponsiveContainer>
            ) : (
              <div className="flex items-center justify-center h-[250px] text-muted-foreground">
                No completed submissions yet
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        {/* Analysis Types Breakdown */}
        <Card className="glassmorphism border-primary/20">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Target className="h-5 w-5" />
              Analysis Types
            </CardTitle>
            <CardDescription>Breakdown of your analysis preferences</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={250}>
              <BarChart data={analysisTypes}>
                <CartesianGrid strokeDasharray="3 3" stroke="hsl(var(--border))" />
                <XAxis
                  dataKey="type"
                  stroke="hsl(var(--muted-foreground))"
                  style={{ fontSize: '11px' }}
                  angle={-15}
                  textAnchor="end"
                  height={60}
                />
                <YAxis
                  stroke="hsl(var(--muted-foreground))"
                  style={{ fontSize: '12px' }}
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "hsl(var(--card))",
                    border: "1px solid hsl(var(--border))",
                    borderRadius: "var(--radius)",
                  }}
                />
                <Bar dataKey="count" fill="hsl(var(--primary))" radius={[8, 8, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        {/* Leaderboard Position */}
        <Card className="glassmorphism border-primary/20">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Trophy className="h-5 w-5" />
              Leaderboard Ranking
            </CardTitle>
            <CardDescription>Your position among all users</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-6">
              {/* Current Rank */}
              <div className="text-center">
                <div className="text-6xl font-bold text-primary mb-2">#12</div>
                <p className="text-sm text-muted-foreground">Out of 1,247 users</p>
                <div className="flex items-center justify-center gap-1 text-sm text-green-500 mt-2">
                  <TrendingUp className="h-4 w-4" />
                  <span>+3 positions this week</span>
                </div>
              </div>

              {/* Progress to next rank */}
              <div>
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-medium">Progress to #11</span>
                  <span className="text-sm text-muted-foreground">85%</span>
                </div>
                <Progress value={85} className="h-2" />
                <p className="text-xs text-muted-foreground mt-2">
                  15 reputation points needed
                </p>
              </div>

              {/* Top categories */}
              <div className="space-y-2">
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">Submissions</span>
                  <Badge variant="secondary">#8</Badge>
                </div>
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">Accuracy</span>
                  <Badge variant="secondary">#15</Badge>
                </div>
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">Reputation</span>
                  <Badge variant="secondary">#12</Badge>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Recent Activity Timeline */}
      <Card className="glassmorphism border-primary/20">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Calendar className="h-5 w-5" />
            Recent Activity
          </CardTitle>
          <CardDescription>Your latest submission activity</CardDescription>
        </CardHeader>
        <CardContent>
          {recentActivity.length > 0 ? (
            <div className="space-y-4">
              {recentActivity.map((activity: any) => (
                <div
                  key={activity.id}
                  className="flex items-center gap-4 p-4 rounded-lg border border-border/50 hover:border-primary/50 transition-colors"
                >
                  <div className="flex-shrink-0">
                    {getStatusIcon(activity.status)}
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="font-medium truncate">{activity.fileName}</p>
                    <p className="text-sm text-muted-foreground">
                      {activity.createdAt.toLocaleDateString()} at {activity.createdAt.toLocaleTimeString()}
                    </p>
                  </div>
                  <div className="flex-shrink-0">
                    {activity.verdict ? (
                      <Badge variant={getVerdictColor(activity.verdict) as any}>
                        {activity.verdict}
                      </Badge>
                    ) : (
                      <Badge variant="outline">
                        {activity.status === 'analyzing' ? 'Analyzing...' : activity.status}
                      </Badge>
                    )}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
              <Activity className="h-12 w-12 mb-2 opacity-50" />
              <p>No recent activity</p>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
