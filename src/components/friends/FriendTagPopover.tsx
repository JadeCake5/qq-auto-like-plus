import { useState, useEffect, useRef, useMemo } from "react";
import { toast } from "sonner";
import { useFriendsStore } from "@/stores/useFriendsStore";

interface FriendTagPopoverProps {
  friendId: number;
  friendTags: { id: number }[];
  children: React.ReactNode;
}

export default function FriendTagPopover({
  friendId,
  friendTags,
  children,
}: FriendTagPopoverProps) {
  const tags = useFriendsStore((s) => s.tags);
  const setFriendTags = useFriendsStore((s) => s.setFriendTags);

  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  const selectedIds = useMemo(() => friendTags.map((t) => t.id), [friendTags]);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    if (open) {
      document.addEventListener("mousedown", handleClickOutside);
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [open]);

  const toggle = async (tagId: number) => {
    const next = selectedIds.includes(tagId)
      ? selectedIds.filter((id) => id !== tagId)
      : [...selectedIds, tagId];
    try {
      await setFriendTags(friendId, next);
      toast.success("标签已更新");
    } catch {
      toast.error("标签更新失败");
    }
  };

  return (
    <div className="relative" ref={ref}>
      <div onClick={() => setOpen(!open)} className="cursor-pointer">
        {children}
      </div>
      {open && (
        <div className="absolute top-full mt-1 left-0 z-10 min-w-[140px] rounded-lg border border-border bg-bg-card shadow-lg py-1">
          {tags.map((tag) => (
            <button
              key={tag.id}
              onClick={() => toggle(tag.id)}
              className="flex items-center gap-2 w-full px-3 py-1.5 text-sm hover:bg-bg-elevated/50 transition-colors"
            >
              <span
                className="w-3 h-3 rounded-sm shrink-0 border"
                style={{
                  backgroundColor: selectedIds.includes(tag.id)
                    ? tag.color
                    : "transparent",
                  borderColor: tag.color,
                }}
              />
              <span className="text-text-primary">{tag.name}</span>
              {selectedIds.includes(tag.id) && (
                <span className="ml-auto text-accent-primary text-xs">✓</span>
              )}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
