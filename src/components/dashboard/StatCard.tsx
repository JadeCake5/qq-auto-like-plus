interface StatCardProps {
  title: string;
  value: string | number;
  gradientFrom: string;
  gradientTo: string;
  icon?: string;
}

export default function StatCard({
  title,
  value,
  gradientFrom,
  gradientTo,
  icon,
}: StatCardProps) {
  return (
    <div
      className="rounded-lg px-4 py-3"
      style={{
        background: `linear-gradient(135deg, ${gradientFrom}, ${gradientTo})`,
      }}
    >
      {icon && (
        <span className="mb-1 block text-base leading-none" role="img" aria-hidden="true">
          {icon}
        </span>
      )}
      <p
        className="font-bold leading-tight text-bg-base"
        style={{ fontSize: "var(--text-stat)" }}
      >
        {value}
      </p>
      <p
        className="mt-0.5 font-medium text-bg-base/80"
        style={{ fontSize: "var(--text-stat-label)" }}
      >
        {title}
      </p>
    </div>
  );
}
