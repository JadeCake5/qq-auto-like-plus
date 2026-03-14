import { NavLink } from "react-router-dom";
import {
  LayoutDashboard,
  Users,
  BarChart3,
  FileText,
  Settings,
} from "lucide-react";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

const topNavItems = [
  { to: "/dashboard", icon: LayoutDashboard, label: "仪表盘" },
  { to: "/friends", icon: Users, label: "好友管理" },
  { to: "/statistics", icon: BarChart3, label: "数据统计" },
  { to: "/logs", icon: FileText, label: "运行日志" },
];

const bottomNavItems = [
  { to: "/settings", icon: Settings, label: "设置" },
];

function NavItem({
  to,
  icon: Icon,
  label,
}: {
  to: string;
  icon: React.ComponentType<{ className?: string }>;
  label: string;
}) {
  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <NavLink
            to={to}
            aria-label={label}
            className={({ isActive }) =>
              cn(
                "relative flex items-center justify-center w-10 h-10 rounded-xl transition-all duration-200 outline-none focus-visible:ring-2 focus-visible:ring-ring",
                isActive
                  ? "bg-gradient-to-br from-primary/20 to-accent/20 text-primary shadow-[0_0_12px_rgba(242,167,195,0.3)]"
                  : "text-text-muted hover:text-text-secondary hover:bg-white/5",
              )
            }
          />
        }
      >
        <Icon className="h-5 w-5" />
      </TooltipTrigger>
      <TooltipContent side="right">
        <p>{label}</p>
      </TooltipContent>
    </Tooltip>
  );
}

export function SidebarNav() {
  return (
    <nav
      className="flex h-full w-14 flex-col items-center gap-2 bg-bg-nav py-4"
      aria-label="主导航"
    >
      {topNavItems.map((item) => (
        <NavItem key={item.to} {...item} />
      ))}
      <div className="mt-auto">
        {bottomNavItems.map((item) => (
          <NavItem key={item.to} {...item} />
        ))}
      </div>
    </nav>
  );
}
