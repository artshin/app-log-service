import { cn } from "@/lib/utils"

const LEVEL_STYLES: Record<string, { dot: string; text: string }> = {
  trace: { dot: "bg-gray-400", text: "text-gray-500 dark:text-gray-400" },
  debug: { dot: "bg-gray-400", text: "text-gray-500 dark:text-gray-400" },
  info: {
    dot: "bg-emerald-500",
    text: "text-emerald-600 dark:text-emerald-400",
  },
  notice: { dot: "bg-sky-500", text: "text-sky-600 dark:text-sky-400" },
  warning: {
    dot: "bg-amber-500",
    text: "text-amber-600 dark:text-amber-400",
  },
  error: { dot: "bg-red-500", text: "text-red-600 dark:text-red-400" },
  critical: {
    dot: "bg-fuchsia-500",
    text: "text-fuchsia-600 dark:text-fuchsia-400",
  },
}

export function LevelBadge({ level }: { level: string }) {
  const s = LEVEL_STYLES[level.toLowerCase()] ?? LEVEL_STYLES.info
  return (
    <span className={cn("inline-flex items-center gap-1.5 text-xs font-medium", s.text)}>
      <span className={cn("h-2 w-2 shrink-0 rounded-full", s.dot)} />
      {level.toUpperCase()}
    </span>
  )
}

export function getLevelDotClass(level: string) {
  return (LEVEL_STYLES[level.toLowerCase()] ?? LEVEL_STYLES.info).dot
}
