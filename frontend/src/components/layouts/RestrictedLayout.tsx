import { Link, Outlet } from "@tanstack/react-router";
import {
  LayoutDashboard,
  ScrollText,
  Globe,
  Server,
  Settings,
  LogOut,
  Sun,
  Moon,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { useTheme } from "@/hooks/use-theme";

const navItems = [
  { to: "/dashboard", label: "Dashboard", icon: LayoutDashboard },
  { to: "/node", label: "Node", icon: Server },
  { to: "/network", label: "Network", icon: Globe },
  { to: "/logs", label: "Logs", icon: ScrollText },
  { to: "/settings", label: "Settings", icon: Settings },
] as const;

export default function RestrictedLayout() {
  const { theme, toggleTheme } = useTheme();

  return (
    <div className="flex min-h-svh flex-col">
      <header className="sticky top-0 z-50 border-b bg-background/95 backdrop-blur supports-backdrop-filter:bg-background/60">
        <div className="flex h-14 items-center px-4 md:px-6">
          <Link
            to="/dashboard"
            className="mr-6 flex items-center gap-2 font-semibold"
          >
            BitNode Console
          </Link>
          <Separator orientation="vertical" className="mr-4 h-6" />
          <nav className="flex items-center gap-1">
            {navItems.map(({ to, label, icon: Icon }) => (
              <Button key={to} variant="ghost" size="sm" asChild>
                <Link
                  to={to}
                  activeProps={{ className: "bg-muted text-foreground" }}
                  inactiveProps={{ className: "text-muted-foreground" }}
                >
                  <Icon data-icon="inline-start" />
                  {label}
                </Link>
              </Button>
            ))}
          </nav>
          <div className="ml-auto flex items-center gap-1">
            <Button variant="ghost" size="icon" onClick={toggleTheme}>
              {theme === "dark" ? <Sun /> : <Moon />}
            </Button>
            <Button variant="ghost" size="sm" asChild>
              <Link to="/auth/logged-out">
                <LogOut data-icon="inline-start" />
                Logout
              </Link>
            </Button>
          </div>
        </div>
      </header>
      <main className="flex-1 p-4 md:p-6">
        <Outlet />
      </main>
    </div>
  );
}
