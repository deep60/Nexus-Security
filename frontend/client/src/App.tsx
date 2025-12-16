import { Switch, Route } from "wouter";
import { queryClient } from "./lib/queryClient";
import { QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "@/components/ui/toaster";
import { TooltipProvider } from "@/components/ui/tooltip";
import { AuthProvider } from "@/contexts/auth-context";
import { NotificationProvider } from "@/components/notification-provider";
import { ProtectedRoute } from "@/components/protected-route";
import Home from "@/pages/home";
import Dashboard from "@/pages/dashboard";
import Marketplace from "@/pages/marketplace";
import ApiDocs from "@/pages/api-docs";
import Login from "@/pages/login";
import Register from "@/pages/register";
import Profile from "@/pages/profile";
import AnalysisDetails from "@/pages/analysis-details";
import NotFound from "@/pages/not-found";

function Router() {
  return (
    <Switch>
      <Route path="/" component={Home} />
      <Route path="/login" component={Login} />
      <Route path="/register" component={Register} />
      <Route path="/dashboard" component={Dashboard} />
      <Route path="/marketplace" component={Marketplace} />
      <Route path="/analysis/:id" component={AnalysisDetails} />
      <Route path="/api" component={ApiDocs} />
      <Route path="/profile">
        <ProtectedRoute>
          <Profile />
        </ProtectedRoute>
      </Route>
      <Route component={NotFound} />
    </Switch>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <NotificationProvider>
          <TooltipProvider>
            <Toaster />
            <Router />
          </TooltipProvider>
        </NotificationProvider>
      </AuthProvider>
    </QueryClientProvider>
  );
}

export default App;
