import { useState } from "react";
import { Dialog } from "@base-ui/react/dialog";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { useFriendsStore } from "@/stores/useFriendsStore";
import { toast } from "sonner";
import type { TagInfo } from "@/types/friends";

const PRESET_COLORS = [
  "#f2a7c3", // 樱花粉
  "#a7c7f2", // 天空蓝
  "#c3a7f2", // 薰衣草紫
  "#a7f2d4", // 薄荷绿
  "#f2cfa7", // 蜜桃橙
  "#f28b8b", // 珊瑚红
  "#9b95a8", // 灰紫
  "#a7f2f0", // 水蓝
];

const PRIORITIES = [
  { value: "high", label: "高" },
  { value: "medium", label: "中" },
  { value: "low", label: "低" },
] as const;

interface TagEditDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  tag?: TagInfo | null;
  existingNames: string[];
  onSave: (name: string, color: string) => Promise<boolean>;
}

function TagEditForm({
  tag,
  existingNames,
  onSave,
  onClose,
}: {
  tag?: TagInfo | null;
  existingNames: string[];
  onSave: (name: string, color: string) => Promise<boolean>;
  onClose: () => void;
}) {
  const isEdit = !!tag;
  const isSystem = tag?.isSystem ?? false;

  const [name, setName] = useState(tag?.name ?? "");
  const [color, setColor] = useState(tag?.color ?? PRESET_COLORS[0]);
  const [priority, setPriority] = useState(tag?.priority ?? "medium");
  const [likeTimes, setLikeTimes] = useState<string>(
    tag?.likeTimes != null ? String(tag.likeTimes) : ""
  );
  const [autoLike, setAutoLike] = useState(tag?.autoLike ?? true);
  const [autoReply, setAutoReply] = useState(tag?.autoReply ?? true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  const updateTagStrategy = useFriendsStore((s) => s.updateTagStrategy);

  const validate = (val: string) => {
    if (!val.trim()) return "标签名称不能为空";
    const others = existingNames.filter((n) => (isEdit ? n !== tag?.name : true));
    if (others.includes(val.trim())) return "标签名称已存在";
    return "";
  };

  const handleSave = async () => {
    const err = validate(name);
    if (err) {
      setError(err);
      return;
    }
    setSaving(true);

    // Step 1: 保存名称/颜色
    const ok = await onSave(name.trim(), color);
    if (!ok) {
      setSaving(false);
      return;
    }

    // Step 2: 保存策略（仅编辑模式）
    if (isEdit && tag) {
      const parsedTimes = likeTimes.trim() === "" ? null : parseInt(likeTimes, 10);
      if (parsedTimes !== null && (isNaN(parsedTimes) || parsedTimes < 1 || parsedTimes > 20)) {
        setError("点赞次数必须在 1-20 之间");
        setSaving(false);
        return;
      }
      const result = await updateTagStrategy(tag.id, parsedTimes, priority, autoLike, autoReply);
      if (!result) {
        toast.error("策略保存失败");
        setSaving(false);
        return;
      }
    }

    setSaving(false);
    onClose();
  };

  return (
    <>
      <Dialog.Title className="text-base font-semibold text-text-primary mb-4">
        {isEdit ? "编辑标签" : "新建标签"}
      </Dialog.Title>

      <div className="space-y-4">
        <div>
          <label className="text-sm text-text-secondary mb-1.5 block">
            标签名称
          </label>
          <Input
            value={name}
            onChange={(e) => {
              setName(e.target.value);
              setError("");
            }}
            placeholder="输入标签名称"
            disabled={isSystem}
            maxLength={20}
          />
          {error && (
            <p className="text-xs text-[#f28b8b] mt-1">{error}</p>
          )}
        </div>

        <div>
          <label className="text-sm text-text-secondary mb-1.5 block">
            选择颜色
          </label>
          <div className="flex flex-wrap gap-2">
            {PRESET_COLORS.map((c) => (
              <button
                key={c}
                type="button"
                onClick={() => setColor(c)}
                className="w-8 h-8 rounded-full transition-all"
                style={{
                  backgroundColor: c,
                  boxShadow:
                    color === c
                      ? `0 0 0 2px var(--color-bg-card), 0 0 0 4px ${c}`
                      : "none",
                }}
              />
            ))}
          </div>
        </div>

        {isEdit && (
          <>
            <div className="border-t border-border pt-4">
              <label className="text-sm font-medium text-text-primary mb-3 block">
                点赞策略
              </label>

              <div className="space-y-3">
                <div>
                  <label className="text-sm text-text-secondary mb-1.5 block">
                    优先级
                  </label>
                  <div className="flex gap-1">
                    {PRIORITIES.map((p) => (
                      <button
                        key={p.value}
                        type="button"
                        onClick={() => setPriority(p.value)}
                        className={`flex-1 px-3 py-1.5 rounded-lg text-sm font-medium transition-all ${
                          priority === p.value
                            ? "bg-primary text-primary-foreground"
                            : "bg-bg-elevated text-text-secondary hover:text-text-primary"
                        }`}
                      >
                        {p.label}
                      </button>
                    ))}
                  </div>
                </div>

                <div>
                  <label className="text-sm text-text-secondary mb-1.5 block">
                    点赞次数
                  </label>
                  <Input
                    type="number"
                    min={1}
                    max={20}
                    value={likeTimes}
                    onChange={(e) => setLikeTimes(e.target.value)}
                    placeholder="使用全局默认"
                  />
                </div>

                <div className="flex items-center justify-between">
                  <label className="text-sm text-text-secondary">
                    参与定时点赞
                  </label>
                  <Switch
                    checked={autoLike}
                    onCheckedChange={setAutoLike}
                    size="sm"
                  />
                </div>

                <div className="flex items-center justify-between">
                  <label className="text-sm text-text-secondary">
                    参与回赞
                  </label>
                  <Switch
                    checked={autoReply}
                    onCheckedChange={setAutoReply}
                    size="sm"
                  />
                </div>

                {isSystem && tag?.name === "不赞" && (
                  <p className="text-xs text-text-tertiary">
                    系统标签"不赞"默认关闭点赞和回赞
                  </p>
                )}
              </div>
            </div>
          </>
        )}
      </div>

      <div className="flex justify-end gap-2 mt-5">
        <Button variant="outline" size="sm" onClick={onClose}>
          取消
        </Button>
        <Button size="sm" onClick={handleSave} disabled={saving}>
          {saving ? "保存中…" : "保存"}
        </Button>
      </div>
    </>
  );
}

export default function TagEditDialog({
  open,
  onOpenChange,
  tag,
  existingNames,
  onSave,
}: TagEditDialogProps) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Backdrop className="fixed inset-0 z-50 bg-black/40" />
        <Dialog.Popup className="fixed left-1/2 top-1/2 z-50 w-[340px] -translate-x-1/2 -translate-y-1/2 rounded-2xl bg-bg-card border border-border p-5 shadow-xl animate-in fade-in zoom-in-95 duration-200 max-h-[90vh] overflow-y-auto">
          {open && (
            <TagEditForm
              key={tag?.id ?? "new"}
              tag={tag}
              existingNames={existingNames}
              onSave={onSave}
              onClose={() => onOpenChange(false)}
            />
          )}
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
