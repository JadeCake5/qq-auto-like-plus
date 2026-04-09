import { useState, useEffect, useRef, useMemo, useCallback } from "react";
import { createPortal } from "react-dom";
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
  const triggerRef = useRef<HTMLDivElement>(null);
  const popoverRef = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState({ top: 0, left: 0 });

  const selectedIds = useMemo(() => friendTags.map((t) => t.id), [friendTags]);

  const updatePosition = useCallback(() => {
    if (!triggerRef.current) return;
    const rect = triggerRef.current.getBoundingClientRect();
    const spaceBelow = window.innerHeight - rect.bottom;
    const popoverHeight = 200; // 估算高度
    // 空间不够则向上弹出
    if (spaceBelow < popoverHeight) {
      setPos({ top: rect.top - popoverHeight - 4, left: rect.left });
    } else {
      setPos({ top: rect.bottom + 4, left: rect.left });
    }
  }, []);

  useEffect(() => {
    if (!open) return;
    updatePosition();

    function handleClickOutside(e: MouseEvent) {
      if (
        triggerRef.current?.contains(e.target as Node) ||
        popoverRef.current?.contains(e.target as Node)
      ) {
        return;
      }
      setOpen(false);
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [open, updatePosition]);

  const toggle = async (tagId: number) => {
    // 单选：点已选中的标签 → 清空；点其他 → 替换为仅这一个
    const next = selectedIds.includes(tagId) ? [] : [tagId];
    try {
      await setFriendTags(friendId, next);
      setOpen(false);
      toast.success("标签已更新");
    } catch {
      toast.error("标签更新失败");
    }
  };

  return (
    <>
      <div
        ref={triggerRef}
        onClick={() => setOpen(!open)}
        className="cursor-pointer"
      >
        {children}
      </div>
      {open &&
        createPortal(
          <div
            ref={popoverRef}
            className="fixed z-50 min-w-[140px] max-h-[200px] overflow-y-auto rounded-lg border border-border bg-bg-card shadow-lg py-1"
            style={{ top: pos.top, left: pos.left }}
          >
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
          </div>,
          document.body
        )}
    </>
  );
}
