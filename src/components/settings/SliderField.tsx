import { useState, useEffect } from "react";
import { Slider } from "@/components/ui/slider";

interface SliderFieldProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  unit?: string;
  onChange: (value: number) => void;
}

export function SliderField({
  label,
  value,
  min,
  max,
  step = 1,
  unit,
  onChange,
}: SliderFieldProps) {
  const [localValue, setLocalValue] = useState(value);

  useEffect(() => {
    setLocalValue(value);
  }, [value]);

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <span className="text-sm text-text-secondary">{label}</span>
        <span className="text-sm font-medium text-text-primary">
          {localValue}
          {unit && <span className="ml-0.5 text-text-muted">{unit}</span>}
        </span>
      </div>
      <Slider
        value={[localValue]}
        min={min}
        max={max}
        step={step}
        onValueChange={(val) => {
          const v = Array.isArray(val) ? val[0] : val;
          setLocalValue(v);
        }}
        onValueCommitted={(val) => {
          const v = Array.isArray(val) ? val[0] : val;
          onChange(Math.min(Math.max(v, min), max));
        }}
        aria-label={label}
      />
    </div>
  );
}
