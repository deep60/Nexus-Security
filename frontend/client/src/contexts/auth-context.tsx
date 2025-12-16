import { createContext, useContext, useState, useEffect, ReactNode } from "react";
import { useToast } from "@/hooks/use-toast";

interface User {
  id: number;
  username: string;
  email: string;
  walletAddress?: string;
  reputation: number;
  createdAt: string;
}

interface AuthContextType {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (email: string, password: string) => Promise<void>;
  register: (username: string, email: string, password: string) => Promise<void>;
  logout: () => void;
  connectWallet: () => Promise<void>;
  disconnectWallet: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const { toast } = useToast();

  useEffect(() => {
    checkAuthStatus();
  }, []);

  const checkAuthStatus = async () => {
    try {
      const sessionId = localStorage.getItem("sessionId");
      if (!sessionId) {
        setIsLoading(false);
        return;
      }

      // Validate session with backend
      const response = await fetch("/api/auth/me", {
        headers: {
          Authorization: `Bearer ${sessionId}`,
        },
      });

      if (response.ok) {
        const userData = await response.json();
        setUser(userData);
      } else {
        // Session expired or invalid
        localStorage.removeItem("sessionId");
        localStorage.removeItem("user");
      }
    } catch (error) {
      console.error("Auth check failed:", error);
      localStorage.removeItem("sessionId");
      localStorage.removeItem("user");
    } finally {
      setIsLoading(false);
    }
  };

  const login = async (email: string, password: string) => {
    try {
      const response = await fetch("/api/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, password }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Login failed");
      }

      const data = await response.json();
      setUser(data.user);
      localStorage.setItem("sessionId", data.sessionId);
      localStorage.setItem("user", JSON.stringify(data.user));

      toast({
        title: "Welcome back!",
        description: `Logged in as ${data.user.username}`,
      });
    } catch (error: any) {
      toast({
        title: "Login failed",
        description: error.message || "Invalid email or password",
        variant: "destructive",
      });
      throw error;
    }
  };

  const register = async (username: string, email: string, password: string) => {
    try {
      const response = await fetch("/api/auth/register", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username, email, password }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Registration failed");
      }

      const data = await response.json();
      setUser(data.user);
      localStorage.setItem("sessionId", data.sessionId);
      localStorage.setItem("user", JSON.stringify(data.user));

      toast({
        title: "Account created!",
        description: "Welcome to Nexus-Security",
      });
    } catch (error: any) {
      toast({
        title: "Registration failed",
        description: error.message || "Email may already be in use",
        variant: "destructive",
      });
      throw error;
    }
  };

  const logout = async () => {
    try {
      const sessionId = localStorage.getItem("sessionId");
      if (sessionId) {
        await fetch("/api/auth/logout", {
          method: "POST",
          headers: {
            Authorization: `Bearer ${sessionId}`,
          },
        });
      }
    } catch (error) {
      console.error("Logout error:", error);
    } finally {
      setUser(null);
      localStorage.removeItem("sessionId");
      localStorage.removeItem("user");
      toast({
        title: "Logged out",
        description: "See you soon!",
      });
    }
  };

  const connectWallet = async () => {
    try {
      if (typeof window.ethereum === "undefined") {
        toast({
          title: "MetaMask not found",
          description: "Please install MetaMask to connect your wallet",
          variant: "destructive",
        });
        return;
      }

      const accounts = await window.ethereum.request({
        method: "eth_requestAccounts",
      });

      const walletAddress = accounts[0];

      // Update wallet address via API
      const sessionId = localStorage.getItem("sessionId");
      if (sessionId) {
        const response = await fetch("/api/auth/wallet", {
          method: "PATCH",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${sessionId}`,
          },
          body: JSON.stringify({ walletAddress }),
        });

        if (response.ok) {
          const updatedUser = await response.json();
          setUser(updatedUser);
          localStorage.setItem("user", JSON.stringify(updatedUser));
        }
      }

      toast({
        title: "Wallet connected",
        description: `Connected: ${walletAddress.substring(0, 6)}...${walletAddress.substring(38)}`,
      });
    } catch (error) {
      toast({
        title: "Connection failed",
        description: "Could not connect to wallet",
        variant: "destructive",
      });
    }
  };

  const disconnectWallet = async () => {
    try {
      const sessionId = localStorage.getItem("sessionId");
      if (sessionId) {
        const response = await fetch("/api/auth/wallet", {
          method: "PATCH",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${sessionId}`,
          },
          body: JSON.stringify({ walletAddress: null }),
        });

        if (response.ok) {
          const updatedUser = await response.json();
          setUser(updatedUser);
          localStorage.setItem("user", JSON.stringify(updatedUser));

          toast({
            title: "Wallet disconnected",
            description: "Your wallet has been disconnected",
          });
        }
      }
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to disconnect wallet",
        variant: "destructive",
      });
    }
  };

  return (
    <AuthContext.Provider
      value={{
        user,
        isAuthenticated: !!user,
        isLoading,
        login,
        register,
        logout,
        connectWallet,
        disconnectWallet,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}

// Extend Window interface for TypeScript
declare global {
  interface Window {
    ethereum?: any;
  }
}
