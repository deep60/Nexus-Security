import { createContext, useContext, useState, useEffect, ReactNode } from "react";
import { useToast } from "@/hooks/use-toast";

interface User {
  id: string;
  username: string;
  email: string;
  walletAddress?: string | null;
  reputationScore: number;
  totalEarnings: string;
  isVerified: boolean;
  createdAt: string;
}

interface AuthContextType {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (identifier: string, password: string) => Promise<void>;
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

/**
 * Unwrap the standard ApiResponse envelope.
 *
 * Backend returns:
 *   { success: bool, data: T | null, message: string | null, timestamp }
 *
 * On error the response status is non-2xx and `success` is false.
 */
async function unwrapApiResponse<T>(response: Response): Promise<T> {
  const body = await response.json();

  if (!response.ok || !body.success) {
    const msg = body.message || body.error || `Request failed (${response.status})`;
    throw new Error(msg);
  }

  return body.data as T;
}

/** Shape of the auth response from /api/v1/auth/login and /register */
interface AuthResponseData {
  user: User;
  access_token: string;
  refresh_token: string;
  expires_in: number;
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

      // Validate token and fetch current user profile
      const response = await fetch("/api/v1/auth/verify", {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (response.ok) {
        const userData = await unwrapApiResponse<User>(response);
        setUser(userData);
      } else {
        // Token expired or invalid
        localStorage.removeItem("token");
        localStorage.removeItem("refreshToken");
        localStorage.removeItem("user");
      }
    } catch (error) {
      console.error("Auth check failed:", error);
      localStorage.removeItem("token");
      localStorage.removeItem("refreshToken");
      localStorage.removeItem("user");
    } finally {
      setIsLoading(false);
    }
  };

  const login = async (identifier: string, password: string) => {
    try {
      const response = await fetch("/api/v1/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ identifier, password }),
      });

      const authData = await unwrapApiResponse<AuthResponseData>(response);

      setUser(authData.user);
      localStorage.setItem("token", authData.access_token);
      localStorage.setItem("refreshToken", authData.refresh_token);
      localStorage.setItem("user", JSON.stringify(authData.user));

      toast({
        title: "Welcome back!",
        description: `Logged in as ${authData.user.username}`,
      });
    } catch (error: any) {
      toast({
        title: "Login failed",
        description: error.message || "Invalid credentials",
        variant: "destructive",
      });
      throw error;
    }
  };

  const register = async (username: string, email: string, password: string) => {
    try {
      const response = await fetch("/api/v1/auth/register", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username, email, password }),
      });

      const authData = await unwrapApiResponse<AuthResponseData>(response);

      setUser(authData.user);
      localStorage.setItem("token", authData.access_token);
      localStorage.setItem("refreshToken", authData.refresh_token);
      localStorage.setItem("user", JSON.stringify(authData.user));

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
        await fetch("/api/v1/auth/logout", {
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
      localStorage.removeItem("refreshToken");
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

      // Sign a message to prove wallet ownership
      const message = `Connect wallet ${walletAddress} to Nexus-Security at ${Date.now()}`;
      const signature = await window.ethereum.request({
        method: "personal_sign",
        params: [message, walletAddress],
      });

      // Update wallet address via API
      const token = getAuthToken();
      if (token) {
        const response = await fetch("/api/v1/auth/wallet/connect", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${token}`,
          },
          body: JSON.stringify({
            wallet_address: walletAddress,
            signature,
            message,
          }),
        });

        if (response.ok) {
          const updatedUser = await unwrapApiResponse<User>(response);
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
      const token = getAuthToken();
      if (token) {
        const response = await fetch("/api/v1/auth/wallet/disconnect", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${token}`,
          },
        });

        if (response.ok) {
          const updatedUser = await unwrapApiResponse<User>(response);
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
