import { Outlet, Link, useLocation } from "react-router-dom";
import { cn } from "../../lib/utils";

const navItems = [
  { path: "/", label: "Welcome" },
  { path: "/providers", label: "Providers" },
  { path: "/agents", label: "Agents" },
  { path: "/projects", label: "Projects" },
];

export function ProjectLayout() {
  const location = useLocation();

  return (
    <div className="flex h-screen bg-background">
      {/* Sidebar */}
      <aside className="w-16 border-r border-border bg-card flex flex-col items-center py-4 gap-2">
        <nav className="flex flex-col gap-1">
          {navItems.map((item) => (
            <Link
              key={item.path}
              to={item.path}
              className={cn(
                "p-2 rounded-lg text-sm transition-colors",
                location.pathname === item.path
                  ? "bg-primary text-primary-foreground"
                  : "hover:bg-accent"
              )}
              title={item.label}
            >
              {item.label[0]}
            </Link>
          ))}
        </nav>
      </aside>

      {/* Main content */}
      <main className="flex-1 flex flex-col overflow-hidden">
        <Outlet />
      </main>
    </div>
  );
}