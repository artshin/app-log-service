import { getTagColor } from "@/lib/utils"
import { useTheme } from "./theme-provider"

export function TagBadge({ tag }: { tag: string }) {
  const { resolved } = useTheme()
  const colors = getTagColor(tag, resolved === "dark")
  return (
    <span
      className="inline-flex items-center rounded px-2 py-0.5 text-xs font-medium"
      style={{ backgroundColor: colors.bg, color: colors.text }}
      title={tag}
    >
      {tag}
    </span>
  )
}
