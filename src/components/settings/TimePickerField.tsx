import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface TimePickerFieldProps {
  label: string;
  hour: number;
  minute: number;
  onHourChange: (hour: number) => void;
  onMinuteChange: (minute: number) => void;
}

export function TimePickerField({
  label,
  hour,
  minute,
  onHourChange,
  onMinuteChange,
}: TimePickerFieldProps) {
  return (
    <div className="space-y-2">
      <span className="text-sm text-text-secondary">{label}</span>
      <div className="flex items-center gap-2">
        <Select
          value={String(hour)}
          onValueChange={(val) => onHourChange(Number(val))}
        >
          <SelectTrigger className="w-20" aria-label="小时">
            <SelectValue placeholder="时" />
          </SelectTrigger>
          <SelectContent className="bg-bg-elevated">
            {Array.from({ length: 24 }, (_, i) => (
              <SelectItem key={i} value={String(i)}>
                {String(i).padStart(2, "0")}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <span className="text-text-muted">:</span>
        <Select
          value={String(minute)}
          onValueChange={(val) => onMinuteChange(Number(val))}
        >
          <SelectTrigger className="w-20" aria-label="分钟">
            <SelectValue placeholder="分" />
          </SelectTrigger>
          <SelectContent className="bg-bg-elevated">
            {Array.from({ length: 60 }, (_, i) => (
              <SelectItem key={i} value={String(i)}>
                {String(i).padStart(2, "0")}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
    </div>
  );
}
