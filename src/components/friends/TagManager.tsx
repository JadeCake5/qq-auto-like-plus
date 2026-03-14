import { useState } from "react";
import { toast } from "sonner";
import { useFriendsStore } from "@/stores/useFriendsStore";
import { Button } from "@/components/ui/button";
import TagEditDialog from "@/components/friends/TagEditDialog";
import { Plus, Pencil, Trash2, ChevronDown, ChevronRight } from "lucide-react";
import { Dialog } from "@base-ui/react/dialog";

export default function TagManager() {
  const tags = useFriendsStore((s) => s.tags);
  const createTag = useFriendsStore((s) => s.createTag);
  const updateTag = useFriendsStore((s) => s.updateTag);
  const deleteTagAction = useFriendsStore((s) => s.deleteTag);

  const [collapsed, setCollapsed] = useState(false);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [editingTag, setEditingTag] = useState<(typeof tags)[0] | null>(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState<number | null>(null);

  const handleCreate = () => {
    setEditingTag(null);
    setEditDialogOpen(true);
  };

  const handleEdit = (tag: (typeof tags)[0]) => {
    setEditingTag(tag);
    setEditDialogOpen(true);
  };

  const handleSave = async (name: string, color: string) => {
    if (editingTag) {
      const result = await updateTag(editingTag.id, name, color);
      if (result) {
        toast.success("标签已更新");
        return true;
      }
      toast.error("更新标签失败");
      return false;
    }
    const result = await createTag(name, color);
    if (result) {
      toast.success("标签已创建");
      return true;
    }
    toast.error("创建标签失败");
    return false;
  };

  const handleDelete = async (id: number) => {
    const ok = await deleteTagAction(id);
    if (ok) {
      toast.success("标签已删除");
    } else {
      toast.error("删除标签失败");
    }
    setDeleteConfirmId(null);
  };

  const deleteTarget = tags.find((t) => t.id === deleteConfirmId);

  return (
    <div className="rounded-xl bg-bg-card border border-border">
      <button
        onClick={() => setCollapsed(!collapsed)}
        className="flex items-center justify-between w-full px-4 py-2.5 text-sm font-medium text-text-primary hover:bg-bg-elevated/50 rounded-xl transition-colors"
      >
        <span className="flex items-center gap-1.5">
          {collapsed ? (
            <ChevronRight className="w-3.5 h-3.5" />
          ) : (
            <ChevronDown className="w-3.5 h-3.5" />
          )}
          标签管理
        </span>
        <Button
          variant="ghost"
          size="xs"
          onClick={(e) => {
            e.stopPropagation();
            handleCreate();
          }}
          className="gap-1"
        >
          <Plus className="w-3 h-3" />
          新建标签
        </Button>
      </button>

      {!collapsed && (
        <div className="flex flex-wrap gap-2 px-4 pb-3">
          {tags.map((tag) => (
            <div
              key={tag.id}
              className="group flex items-center gap-1 px-2.5 py-1 rounded-md text-xs text-white/90"
              style={{ backgroundColor: tag.color }}
            >
              <span>{tag.name}</span>
              <button
                onClick={() => handleEdit(tag)}
                className="opacity-0 group-hover:opacity-100 transition-opacity ml-0.5"
                title="编辑"
              >
                <Pencil className="w-3 h-3" />
              </button>
              {!tag.isSystem && (
                <button
                  onClick={() => setDeleteConfirmId(tag.id)}
                  className="opacity-0 group-hover:opacity-100 transition-opacity"
                  title="删除"
                >
                  <Trash2 className="w-3 h-3" />
                </button>
              )}
            </div>
          ))}
          {tags.length === 0 && (
            <span className="text-xs text-text-secondary">暂无标签</span>
          )}
        </div>
      )}

      <TagEditDialog
        open={editDialogOpen}
        onOpenChange={setEditDialogOpen}
        tag={editingTag}
        existingNames={tags.map((t) => t.name)}
        onSave={handleSave}
      />

      {/* 删除确认对话框 */}
      <Dialog.Root
        open={deleteConfirmId !== null}
        onOpenChange={(open) => {
          if (!open) setDeleteConfirmId(null);
        }}
      >
        <Dialog.Portal>
          <Dialog.Backdrop className="fixed inset-0 z-50 bg-black/40" />
          <Dialog.Popup className="fixed left-1/2 top-1/2 z-50 w-[300px] -translate-x-1/2 -translate-y-1/2 rounded-2xl bg-bg-card border border-border p-5 shadow-xl animate-in fade-in zoom-in-95 duration-200">
            <Dialog.Title className="text-base font-semibold text-text-primary mb-2">
              删除标签
            </Dialog.Title>
            <p className="text-sm text-text-secondary mb-4">
              确定要删除标签「{deleteTarget?.name}」吗？该标签下的好友关联将被移除。
            </p>
            <div className="flex justify-end gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setDeleteConfirmId(null)}
              >
                取消
              </Button>
              <Button
                variant="destructive"
                size="sm"
                onClick={() => deleteConfirmId && handleDelete(deleteConfirmId)}
              >
                删除
              </Button>
            </div>
          </Dialog.Popup>
        </Dialog.Portal>
      </Dialog.Root>
    </div>
  );
}
