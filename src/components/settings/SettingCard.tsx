import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { RotateCcw } from "lucide-react";

interface SettingCardProps {
  title: string;
  icon: ReactNode;
  onResetDefaults: () => void;
  children: ReactNode;
}

export function SettingCard({
  title,
  icon,
  onResetDefaults,
  children,
}: SettingCardProps) {
  return (
    <div className="rounded-[14px] bg-bg-card p-5">
      <div className="mb-4 flex items-center justify-between">
        <div className="flex items-center gap-2">
          {icon}
          <h2 className="text-[length:var(--text-subheading)] font-medium text-text-primary">
            {title}
          </h2>
        </div>
        <Button
          variant="ghost"
          size="sm"
          onClick={onResetDefaults}
          className="text-text-muted hover:text-text-secondary"
          aria-label={`恢复${title}默认设置`}
        >
          <RotateCcw className="mr-1 size-3.5" />
          恢复默认
        </Button>
      </div>
      <div className="space-y-4">{children}</div>
    </div>
  );
}
