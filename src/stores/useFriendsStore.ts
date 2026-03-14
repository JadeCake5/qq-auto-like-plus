import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { FriendWithTags, TagInfo, SyncFriendsResult } from "@/types/friends";

interface FriendsStore {
  friends: FriendWithTags[];
  tags: TagInfo[];
  isLoading: boolean;
  isSyncing: boolean;
  searchQuery: string;
  selectedTagIds: number[];
  setSearchQuery: (q: string) => void;
  setSelectedTagIds: (ids: number[]) => void;
  fetchFriends: () => Promise<void>;
  fetchTags: () => Promise<void>;
  syncFriends: () => Promise<SyncFriendsResult | null>;
  createTag: (name: string, color: string) => Promise<TagInfo | null>;
  updateTag: (id: number, name: string, color: string) => Promise<TagInfo | null>;
  deleteTag: (id: number) => Promise<boolean>;
  setFriendTags: (friendId: number, tagIds: number[]) => Promise<void>;
  updateTagStrategy: (id: number, likeTimes: number | null, priority: string, autoLike: boolean, autoReply: boolean) => Promise<TagInfo | null>;
}

export const useFriendsStore = create<FriendsStore>((set) => ({
  friends: [],
  tags: [],
  isLoading: false,
  isSyncing: false,
  searchQuery: "",
  selectedTagIds: [],
  setSearchQuery: (q) => set({ searchQuery: q }),
  setSelectedTagIds: (ids) => set({ selectedTagIds: ids }),
  fetchFriends: async () => {
    set({ isLoading: true });
    try {
      const friends = await invoke<FriendWithTags[]>("get_friends");
      set({ friends });
    } catch {
      // 静默忽略
    }
    set({ isLoading: false });
  },
  fetchTags: async () => {
    try {
      const tags = await invoke<TagInfo[]>("get_tags");
      set({ tags });
    } catch {
      // 静默忽略
    }
  },
  syncFriends: async () => {
    set({ isSyncing: true });
    try {
      const result = await invoke<SyncFriendsResult>("sync_friends");
      const friends = await invoke<FriendWithTags[]>("get_friends");
      set({ friends, isSyncing: false });
      return result;
    } catch {
      set({ isSyncing: false });
      return null;
    }
  },
  createTag: async (name, color) => {
    try {
      const tag = await invoke<TagInfo>("create_tag", { name, color });
      set((s) => ({ tags: [...s.tags, tag] }));
      return tag;
    } catch {
      return null;
    }
  },
  updateTag: async (id, name, color) => {
    try {
      const tag = await invoke<TagInfo>("update_tag", { id, name, color });
      set((s) => ({
        tags: s.tags.map((t) => (t.id === id ? tag : t)),
        friends: s.friends.map((f) => ({
          ...f,
          tags: f.tags.map((t) => (t.id === id ? tag : t)),
        })),
      }));
      return tag;
    } catch {
      return null;
    }
  },
  deleteTag: async (id) => {
    try {
      await invoke("delete_tag", { id });
      set((s) => ({
        tags: s.tags.filter((t) => t.id !== id),
        friends: s.friends.map((f) => ({
          ...f,
          tags: f.tags.filter((t) => t.id !== id),
        })),
        selectedTagIds: s.selectedTagIds.filter((i) => i !== id),
      }));
      return true;
    } catch {
      return false;
    }
  },
  setFriendTags: async (friendId, tagIds) => {
    try {
      const tags = await invoke<TagInfo[]>("set_friend_tags", { friendId, tagIds });
      set((s) => ({
        friends: s.friends.map((f) =>
          f.userId === friendId ? { ...f, tags } : f
        ),
      }));
    } catch {
      // 静默
    }
  },
  updateTagStrategy: async (id, likeTimes, priority, autoLike, autoReply) => {
    try {
      const tag = await invoke<TagInfo>("update_tag_strategy", {
        id, likeTimes, priority, autoLike, autoReply,
      });
      set((s) => ({
        tags: s.tags.map((t) => (t.id === id ? tag : t)),
        friends: s.friends.map((f) => ({
          ...f,
          tags: f.tags.map((t) => (t.id === id ? tag : t)),
        })),
      }));
      return tag;
    } catch {
      return null;
    }
  },
}));
