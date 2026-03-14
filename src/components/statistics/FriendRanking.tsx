import type { FriendRanking as FriendRankingType } from "@/types/stats";

interface FriendRankingProps {
  ranking: FriendRankingType[];
}

export default function FriendRanking({ ranking }: FriendRankingProps) {
  if (ranking.length === 0) {
    return (
      <div className="flex flex-1 items-center justify-center py-8">
        <span className="text-[length:var(--text-caption)] text-text-muted">
          暂无互动数据
        </span>
      </div>
    );
  }

  const maxLikes = ranking[0]?.totalLikes ?? 1;

  return (
    <div className="flex flex-col gap-2" role="list" aria-label="好友互动排行 TOP 10">
      {ranking.map((friend, i) => (
        <div
          key={friend.userId}
          className="flex items-center gap-2.5"
          role="listitem"
        >
          <span className="w-5 shrink-0 text-center text-[length:var(--text-caption)] font-medium text-text-muted">
            {i + 1}
          </span>
          <div className="flex size-10 shrink-0 items-center justify-center rounded-full bg-bg-elevated text-[length:var(--text-body)] text-text-secondary">
            {friend.nickname.charAt(0) || "?"}
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center justify-between">
              <span className="truncate text-[length:var(--text-caption)] text-text-primary">
                {friend.nickname}
              </span>
              <span className="shrink-0 text-[length:var(--text-caption)] text-text-muted">
                {friend.totalLikes}次
              </span>
            </div>
            <div className="mt-1 h-1.5 overflow-hidden rounded-full bg-bg-elevated">
              <div
                className="h-full rounded-full transition-all duration-300"
                style={{
                  width: `${(friend.totalLikes / Math.max(maxLikes, 1)) * 100}%`,
                  background: "linear-gradient(90deg, #f2a7c3, #c3a7f2)",
                }}
              />
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}
