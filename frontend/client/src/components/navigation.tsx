import { Link, useLocation } from "wouter";
import { Button } from "@/components/ui/button";
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Menu, Shield, User, LogOut, Wallet } from "lucide-react";
import { useAuth } from "@/contexts/auth-context";

export function Navigation() {
  const [location] = useLocation();
  const { user, isAuthenticated, logout, connectWallet } = useAuth();

  const navItems = [
    { href: "/dashboard", label: "Dashboard" },
    { href: "/marketplace", label: "Marketplace" },
    { href: "/api", label: "API" },
  ];

  const userInitials = user?.username
    .split(" ")
    .map((n) => n[0])
    .join("")
    .toUpperCase() || "U";

  return (
    <nav className="relative z-10 glassmorphism border-b border-border/50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between items-center h-16">
          <Link href="/" data-testid="link-home">
            <div className="flex items-center space-x-2 text-2xl font-bold bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent">
              <Shield className="h-8 w-8 text-primary" />
              <span>NEXUS-SECURITY</span>
            </div>
          </Link>

          {/* Desktop Navigation */}
          <div className="hidden md:flex items-center space-x-8">
            {navItems.map((item) => (
              <Link
                key={item.href}
                href={item.href}
                data-testid={`link-${item.label.toLowerCase()}`}
                className={`text-muted-foreground hover:text-primary transition-colors ${
                  location === item.href ? "text-primary" : ""
                }`}
              >
                {item.label}
              </Link>
            ))}

            {isAuthenticated && user ? (
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="ghost" className="relative h-10 w-10 rounded-full">
                    <Avatar className="h-10 w-10 border-2 border-primary">
                      <AvatarFallback className="bg-gradient-to-br from-primary to-secondary">
                        {userInitials}
                      </AvatarFallback>
                    </Avatar>
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="w-56">
                  <DropdownMenuLabel>
                    <div className="flex flex-col space-y-1">
                      <p className="text-sm font-medium">{user.username}</p>
                      <p className="text-xs text-muted-foreground">{user.email}</p>
                    </div>
                  </DropdownMenuLabel>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem asChild>
                    <Link href="/profile">
                      <a className="flex items-center w-full cursor-pointer">
                        <User className="mr-2 h-4 w-4" />
                        <span>Profile</span>
                      </a>
                    </Link>
                  </DropdownMenuItem>
                  {!user.walletAddress && (
                    <DropdownMenuItem onClick={connectWallet}>
                      <Wallet className="mr-2 h-4 w-4" />
                      <span>Connect Wallet</span>
                    </DropdownMenuItem>
                  )}
                  <DropdownMenuSeparator />
                  <DropdownMenuItem onClick={logout}>
                    <LogOut className="mr-2 h-4 w-4" />
                    <span>Log out</span>
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            ) : (
              <div className="flex items-center gap-3">
                <Link href="/login">
                  <Button variant="ghost" data-testid="button-login">
                    Sign In
                  </Button>
                </Link>
                <Link href="/register">
                  <Button className="glow-effect" data-testid="button-register">
                    Get Started
                  </Button>
                </Link>
              </div>
            )}
          </div>

          {/* Mobile Navigation */}
          <Sheet>
            <SheetTrigger asChild>
              <Button variant="ghost" size="icon" className="md:hidden" data-testid="button-menu">
                <Menu className="h-6 w-6" />
              </Button>
            </SheetTrigger>
            <SheetContent>
              <div className="flex flex-col space-y-4 mt-8">
                {isAuthenticated && user && (
                  <div className="flex items-center gap-3 pb-4 border-b border-border">
                    <Avatar className="h-12 w-12 border-2 border-primary">
                      <AvatarFallback className="bg-gradient-to-br from-primary to-secondary">
                        {userInitials}
                      </AvatarFallback>
                    </Avatar>
                    <div>
                      <p className="font-medium">{user.username}</p>
                      <p className="text-sm text-muted-foreground">{user.email}</p>
                    </div>
                  </div>
                )}

                {navItems.map((item) => (
                  <Link
                    key={item.href}
                    href={item.href}
                    data-testid={`mobile-link-${item.label.toLowerCase()}`}
                    className={`text-lg ${
                      location === item.href ? "text-primary" : "text-muted-foreground"
                    }`}
                  >
                    {item.label}
                  </Link>
                ))}

                {isAuthenticated && user ? (
                  <>
                    <Link href="/profile">
                      <Button variant="outline" className="w-full justify-start">
                        <User className="mr-2 h-4 w-4" />
                        Profile
                      </Button>
                    </Link>
                    {!user.walletAddress && (
                      <Button variant="outline" className="w-full justify-start" onClick={connectWallet}>
                        <Wallet className="mr-2 h-4 w-4" />
                        Connect Wallet
                      </Button>
                    )}
                    <Button variant="destructive" className="w-full justify-start" onClick={logout}>
                      <LogOut className="mr-2 h-4 w-4" />
                      Log out
                    </Button>
                  </>
                ) : (
                  <>
                    <Link href="/login">
                      <Button variant="outline" className="w-full" data-testid="mobile-button-login">
                        Sign In
                      </Button>
                    </Link>
                    <Link href="/register">
                      <Button className="w-full glow-effect" data-testid="mobile-button-register">
                        Get Started
                      </Button>
                    </Link>
                  </>
                )}
              </div>
            </SheetContent>
          </Sheet>
        </div>
      </div>
    </nav>
  );
}
