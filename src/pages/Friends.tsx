import { useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";
import { useFriendsStore } from "@/stores/useFriendsStore";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import FriendCard from "@/components/friends/FriendCard";
import TagManager from "@/components/friends/TagManager";
import { RefreshCw, Search, ChevronDown, X } from "lucide-react";

function TagFilterDropdown() {
  const tags = useFriendsStore((s) => s.tags);
  const selectedTagIds = useFriendsStore((s) => s.selectedTagIds);
  const setSelectedTagIds = useFriendsStore((s) => s.setSelectedTagIds);
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const toggle = (id: number) => {
    setSelectedTagIds(
      selectedTagIds.includes(id)
        ? selectedTagIds.filter((i) => i !== id)
        : [...selectedTagIds, id]
    );
  };

  return (
    <div className="relative" ref={ref}>
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1.5 h-9 px-3 rounded-lg border border-border bg-bg-elevated text-sm text-text-primary hover:bg-bg-elevated/80 transition-colors"
      >
        <span>标签筛选</span>
        {selectedTagIds.length > 0 && (
          <span className="bg-accent-primary/20 text-accent-primary text-xs px-1.5 rounded-full">
            {selectedTagIds.length}
          </span>
        )}
        <ChevronDown className="w-3.5 h-3.5 text-text-secondary" />
      </button>
      {selectedTagIds.length > 0 && (
        <button
          onClick={(e) => {
            e.stopPropagation();
            setSelectedTagIds([]);
          }}
          className="absolute -top-1.5 -right-1.5 w-4 h-4 rounded-full bg-text-secondary/60 flex items-center justify-center hover:bg-text-secondary/80"
        >
          <X className="w-2.5 h-2.5 text-white" />
        </button>
      )}
      {open && (
        <div className="absolute top-full mt-1 left-0 z-10 min-w-[160px] rounded-lg border border-border bg-bg-card shadow-lg py-1">
          {tags.map((tag) => (
            <button
              key={tag.id}
              onClick={() => toggle(tag.id)}
              className="flex items-center gap-2 w-full px-3 py-1.5 text-sm hover:bg-bg-elevated/50 transition-colors"
            >
              <span
                className="w-3 h-3 rounded-sm shrink-0"
                style={{ backgroundColor: tag.color }}
              />
              <span className="text-text-primary">{tag.name}</span>
              {selectedTagIds.includes(tag.id) && (
                <span className="ml-auto text-accent-primary">✓</span>
              )}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

export default function Friends() {
  const {
    friends,
    isLoading,
    isSyncing,
    searchQuery,
    selectedTagIds,
    setSearchQuery,
    fetchFriends,
    fetchTags,
    syncFriends,
  } = useFriendsStore();

  const hasSyncedRef = useRef(false);

  useEffect(() => {
    fetchTags();
    fetchFriends().then(() => {
      if (!hasSyncedRef.current) {
        hasSyncedRef.current = true;
        syncFriends().then((result) => {
          if (result) {
            if (result.newCount > 0) {
              toast.success(`同步完成，发现 ${result.newCount} 个新好友`);
            }
          }
        });
      }
    });
  }, [fetchFriends, fetchTags, syncFriends]);

  const filteredFriends = useMemo(() => {
    let result = friends;
    if (searchQuery) {
      const q = searchQuery.toLowerCase();
      result = result.filter(
        (f) =>
          f.nickname.toLowerCase().includes(q) ||
          f.remark.toLowerCase().includes(q)
      );
    }
    if (selectedTagIds.length > 0) {
      result = result.filter((f) =>
        f.tags.some((t) => selectedTagIds.includes(t.id))
      );
    }
    return result;
  }, [friends, searchQuery, selectedTagIds]);

  const handleSync = async () => {
    const result = await syncFriends();
    if (result) {
      toast.success(
        `同步完成：共 ${result.total} 人，新增 ${result.newCount} 人`
      );
    } else {
      toast.error("同步失败，请检查 NapCat 连接");
    }
  };

  return (
    <div className="page-enter flex h-full flex-col gap-3">
      {/* 页面标题 + 操作 */}
      <div className="flex items-center justify-between">
        <h1 className="text-[length:var(--text-display)] font-bold text-text-primary">
          好友管理
        </h1>
        <Button
          variant="outline"
          size="sm"
          onClick={handleSync}
          disabled={isSyncing}
          className="gap-1.5"
        >
          <RefreshCw
            className={`w-3.5 h-3.5 ${isSyncing ? "animate-spin" : ""}`}
          />
          {isSyncing ? "同步中…" : "同步好友"}
        </Button>
      </div>

      {/* 工具栏 */}
      <div className="flex items-center gap-2">
        <div className="relative flex-1 max-w-[240px]">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-text-secondary" />
          <Input
            placeholder="搜索昵称/备注"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-8 h-9"
          />
        </div>
        <TagFilterDropdown />
        <span className="ml-auto text-xs text-text-secondary">
          共 {filteredFriends.length} 人
          {filteredFriends.length !== friends.length &&
            ` / ${friends.length}`}
        </span>
      </div>

      {/* 标签管理 */}
      <TagManager />

      {/* 好友列表 / 空状态 */}
      <div className="flex flex-1 min-h-0 flex-col rounded-[14px] bg-bg-card">
        {isLoading && friends.length === 0 ? (
          <div className="flex flex-1 items-center justify-center">
            <RefreshCw className="w-5 h-5 text-text-secondary animate-spin" />
          </div>
        ) : friends.length === 0 ? (
          <div className="flex flex-1 flex-col items-center justify-center gap-3 py-8">
            <span className="text-[48px]" role="img" aria-label="空状态">
              🐱
            </span>
            <p className="text-[length:var(--text-body)] text-text-secondary">
              还没有好友数据，请先登录 QQ~
            </p>
          </div>
        ) : filteredFriends.length === 0 ? (
          <div className="flex flex-1 flex-col items-center justify-center gap-3 py-8">
            <span className="text-[48px]" role="img" aria-label="无结果">
              🔍
            </span>
            <p className="text-[length:var(--text-body)] text-text-secondary">
              没有匹配的好友
            </p>
          </div>
        ) : (
          <div className="flex-1 overflow-y-auto px-2 py-1">
            {filteredFriends.map((friend) => (
              <div
                key={friend.userId}
                style={{
                  contentVisibility: "auto",
                  containIntrinsicSize: "0 64px",
                }}
              >
                <FriendCard friend={friend} />
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
