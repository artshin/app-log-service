import { getLevelDotClass } from "./level-badge"
import { cn } from "@/lib/utils"

const LEVELS = ["trace", "debug", "info", "notice", "warning", "error", "critical"]

interface StatsBarProps {
  totalCount: number
  levelCounts: Record<string, number>
}

export function StatsBar({ totalCount, levelCounts }: StatsBarProps) {
  return (
    <div className="flex items-center gap-4 text-xs text-muted-foreground px-1">
      <span className="font-medium">
        {totalCount.toLocaleString()} event{totalCount !== 1 ? "s" : ""}
      </span>
      {LEVELS.map((level) => {
        const count = levelCounts[level] ?? 0
        if (count === 0) return null
        return (
          <span key={level} className="inline-flex items-center gap-1">
            <span className={cn("h-2 w-2 rounded-full", getLevelDotClass(level))} />
            <span>{count}</span>
          </span>
        )
      })}
    </div>
  )
}
