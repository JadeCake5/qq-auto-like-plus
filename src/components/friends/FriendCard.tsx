import type { FriendWithTags } from "@/types/friends";
import FriendTagPopover from "@/components/friends/FriendTagPopover";
import { Tag } from "lucide-react";

function AvatarPlaceholder({ nickname }: { nickname: string }) {
  const char = nickname.charAt(0) || "?";
  const hue = (char.charCodeAt(0) * 37) % 360;
  return (
    <div
      className="w-10 h-10 rounded-full flex items-center justify-center text-white text-sm font-medium shrink-0"
      style={{
        background: `linear-gradient(135deg, hsl(${hue}, 60%, 60%), hsl(${(hue + 40) % 360}, 60%, 50%))`,
      }}
    >
      {char}
    </div>
  );
}

export default function FriendCard({ friend }: { friend: FriendWithTags }) {
  return (
    <div className="flex items-center gap-3 px-4 py-3 rounded-lg hover:bg-bg-elevated/50 transition-colors">
      <AvatarPlaceholder nickname={friend.nickname} />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium text-text-primary truncate">
            {friend.nickname}
          </span>
          {friend.remark && friend.remark !== friend.nickname && (
            <span className="text-xs text-text-secondary truncate">
              ({friend.remark})
            </span>
          )}
        </div>
        <FriendTagPopover friendId={friend.userId} friendTags={friend.tags}>
          <div className="flex items-center gap-1.5 mt-0.5 flex-wrap">
            {friend.tags.length > 0 ? (
              friend.tags.map((tag) => (
                <span
                  key={tag.id}
                  className="inline-block px-1.5 py-0.5 rounded-md text-xs leading-none text-white/90"
                  style={{ backgroundColor: tag.color }}
                >
                  {tag.name}
                </span>
              ))
            ) : (
              <span className="inline-flex items-center gap-0.5 text-xs text-text-secondary/60 hover:text-text-secondary transition-colors">
                <Tag className="w-3 h-3" />
                +标签
              </span>
            )}
          </div>
        </FriendTagPopover>
      </div>
      <span
        className={`shrink-0 text-sm font-medium ${
          friend.likedToday ? "text-[#7ecba1]" : "text-text-secondary/40"
        }`}
      >
        {friend.likedToday ? "✓" : "—"}
      </span>
    </div>
  );
}
