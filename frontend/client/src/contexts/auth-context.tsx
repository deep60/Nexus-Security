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

/** Helper – returns the stored JWT (if any) */
export function getAuthToken(): string | null {
  return localStorage.getItem("token");
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const { toast } = useToast();

  useEffect(() => {
    checkAuthStatus();
  }, []);

  const checkAuthStatus = async () => {
    try {
      const token = getAuthToken();
      if (!token) {
        setIsLoading(false);
        return;
      }

      // Validate token with backend
      const response = await fetch("/api/auth/me", {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (response.ok) {
        const userData = await response.json();
        // Handle both { user: ... } and direct user object
        setUser(userData.user || userData);
      } else {
        // Token expired or invalid
        localStorage.removeItem("token");
        localStorage.removeItem("user");
      }
    } catch (error) {
      console.error("Auth check failed:", error);
      localStorage.removeItem("token");
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
        throw new Error(error.error || error.message || "Login failed");
      }

      const data = await response.json();
      const userData = data.user || data;
      const token = data.token || data.sessionId; // accept both JWT and legacy sessionId

      setUser(userData);
      if (token) localStorage.setItem("token", token);
      localStorage.setItem("user", JSON.stringify(userData));

      toast({
        title: "Welcome back!",
        description: `Logged in as ${userData.username}`,
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
        throw new Error(error.error || error.message || "Registration failed");
      }

      const data = await response.json();
      const userData = data.user || data;
      const token = data.token || data.sessionId;

      setUser(userData);
      if (token) localStorage.setItem("token", token);
      localStorage.setItem("user", JSON.stringify(userData));

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
      const token = getAuthToken();
      if (token) {
        await fetch("/api/auth/logout", {
          method: "POST",
          headers: {
            Authorization: `Bearer ${token}`,
          },
        });
      }
    } catch (error) {
      console.error("Logout error:", error);
    } finally {
      setUser(null);
      localStorage.removeItem("token");
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
      const token = getAuthToken();
      if (token) {
        const response = await fetch("/api/auth/wallet/connect", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${token}`,
          },
          body: JSON.stringify({ walletAddress }),
        });

        if (response.ok) {
          const updatedUser = await response.json();
          const userData = updatedUser.user || updatedUser;
          setUser(userData);
          localStorage.setItem("user", JSON.stringify(userData));
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
      const token = getAuthToken();
      if (token) {
        const response = await fetch("/api/auth/wallet/disconnect", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${token}`,
          },
        });

        if (response.ok) {
          const updatedUser = await response.json();
          const userData = updatedUser.user || updatedUser;
          setUser(userData);
          localStorage.setItem("user", JSON.stringify(userData));

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
