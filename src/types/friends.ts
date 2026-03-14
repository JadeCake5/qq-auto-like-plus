export interface TagInfo {
  id: number;
  name: string;
  color: string;
  isSystem: boolean;
  likeTimes: number | null;
  priority: string; // "high" | "medium" | "low"
  autoLike: boolean;
  autoReply: boolean;
}

export interface FriendWithTags {
  userId: number;
  nickname: string;
  remark: string;
  tags: TagInfo[];
  likedToday: boolean;
}

export interface SyncFriendsResult {
  total: number;
  newCount: number;
}

export interface CreateTagParams {
  name: string;
  color: string;
}

export interface UpdateTagParams {
  id: number;
  name: string;
  color: string;
}
