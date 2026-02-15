import { useCallback, useEffect, useRef, useState } from "react"
import {
  Search,
  Filter,
  Columns3,
  ArrowUpDown,
  Download,
  RefreshCw,
  Trash2,
  Moon,
  Sun,
  Monitor,
} from "lucide-react"
import { toast } from "sonner"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Switch } from "@/components/ui/switch"
import { Checkbox } from "@/components/ui/checkbox"
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Separator } from "@/components/ui/separator"
import { Badge } from "@/components/ui/badge"
import { useTheme } from "./theme-provider"
import { TagBadge } from "./tag-badge"
import type { Preferences } from "@/hooks/use-preferences"
import type { ConnectionStatus } from "@/hooks/use-log-stream"
import type { LogEntry } from "@/types/log-entry"
import { cn } from "@/lib/utils"

const ALL_LEVELS = [
  "trace",
  "debug",
  "info",
  "notice",
  "warning",
  "error",
  "critical",
] as const

interface ToolbarProps {
  prefs: Preferences
  onUpdate: (patch: Partial<Preferences>) => void
  sources: string[]
  allTags: string[]
  connectionStatus: ConnectionStatus
  entries: LogEntry[]
  onRefresh: () => void
  onClear: () => void
}

export function Toolbar({
  prefs,
  onUpdate,
  sources,
  allTags,
  connectionStatus,
  entries,
  onRefresh,
  onClear,
}: ToolbarProps) {
  const { theme, setTheme } = useTheme()
  const searchRef = useRef<HTMLInputElement>(null)
  const [searchValue, setSearchValue] = useState(prefs.searchQuery)
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined)

  // Keyboard shortcut: "/" to focus search
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (
        e.key === "/" &&
        !(e.target instanceof HTMLInputElement) &&
        !(e.target instanceof HTMLTextAreaElement)
      ) {
        e.preventDefault()
        searchRef.current?.focus()
      }
    }
    document.addEventListener("keydown", onKey)
    return () => document.removeEventListener("keydown", onKey)
  }, [])

  const handleSearch = useCallback(
    (value: string) => {
      setSearchValue(value)
      clearTimeout(debounceRef.current)
      debounceRef.current = setTimeout(() => {
        onUpdate({ searchQuery: value })
      }, 200)
    },
    [onUpdate],
  )

  const toggleLevel = useCallback(
    (level: string, checked: boolean) => {
      const levels = checked
        ? [...prefs.levels, level]
        : prefs.levels.filter((l) => l !== level)
      onUpdate({ levels })
    },
    [prefs.levels, onUpdate],
  )

  const toggleTag = useCallback(
    (tag: string, checked: boolean) => {
      const tags = checked
        ? [...prefs.tags, tag]
        : prefs.tags.filter((t) => t !== tag)
      onUpdate({ tags })
    },
    [prefs.tags, onUpdate],
  )

  const toggleColumn = useCallback(
    (col: string, visible: boolean) => {
      const hiddenColumns = visible
        ? prefs.hiddenColumns.filter((c) => c !== col)
        : [...prefs.hiddenColumns, col]
      onUpdate({ hiddenColumns })
    },
    [prefs.hiddenColumns, onUpdate],
  )

  const exportJSON = useCallback(() => {
    const json = JSON.stringify(entries, null, 2)
    navigator.clipboard
      .writeText(json)
      .then(() => toast.success("Copied JSON to clipboard"))
      .catch(() => toast.error("Failed to copy"))
  }, [entries])

  const exportTXT = useCallback(() => {
    const lines = entries.map((e) => {
      const ts = new Date(e.timestamp).toISOString()
      return `[${ts}] [${e.level.toUpperCase()}] [${e.source}] ${e.message}`
    })
    navigator.clipboard
      .writeText(lines.join("\n"))
      .then(() => toast.success("Copied text to clipboard"))
      .catch(() => toast.error("Failed to copy"))
  }, [entries])

  const handleClear = useCallback(() => {
    if (window.confirm("Are you sure you want to clear all logs?")) {
      onClear()
    }
  }, [onClear])

  const statusColor =
    connectionStatus === "connected"
      ? "bg-green-500"
      : connectionStatus === "connecting"
        ? "bg-yellow-400"
        : "bg-gray-400"

  const allColumns = ["level", "time", "source", "tags", "caller", "message"]

  return (
    <div className="flex flex-wrap items-center gap-2">
      {/* Search */}
      <div className="relative flex-1 min-w-[200px] max-w-sm">
        <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <Input
          ref={searchRef}
          placeholder="Search logs... (press /)"
          value={searchValue}
          onChange={(e) => handleSearch(e.target.value)}
          className="pl-9 h-8 text-sm"
        />
      </div>

      {/* Level filter */}
      <Popover>
        <PopoverTrigger asChild>
          <Button variant="outline" size="sm" className="h-8 gap-1.5">
            <Filter className="h-3.5 w-3.5" />
            Level
            <Badge variant="secondary" className="h-5 px-1 text-[10px]">
              {prefs.levels.length}
            </Badge>
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-48 p-2" align="start">
          <div className="space-y-1">
            {ALL_LEVELS.map((level) => (
              <label
                key={level}
                className="flex items-center gap-2 px-2 py-1 text-sm rounded hover:bg-accent cursor-pointer"
              >
                <Checkbox
                  checked={prefs.levels.includes(level)}
                  onCheckedChange={(c) => toggleLevel(level, !!c)}
                />
                <span className="capitalize">{level}</span>
              </label>
            ))}
          </div>
          <Separator className="my-2" />
          <div className="flex gap-2 px-2">
            <button
              className="text-xs text-muted-foreground hover:text-foreground"
              onClick={() => onUpdate({ levels: [...ALL_LEVELS] })}
            >
              Select all
            </button>
            <button
              className="text-xs text-muted-foreground hover:text-foreground"
              onClick={() => onUpdate({ levels: [] })}
            >
              Clear all
            </button>
          </div>
        </PopoverContent>
      </Popover>

      {/* Source filter */}
      {sources.length > 0 && (
        <Popover>
          <PopoverTrigger asChild>
            <Button variant="outline" size="sm" className="h-8 gap-1.5">
              <Filter className="h-3.5 w-3.5" />
              {prefs.source || "All Sources"}
            </Button>
          </PopoverTrigger>
          <PopoverContent className="w-48 p-2" align="start">
            <button
              className={cn(
                "w-full text-left px-2 py-1.5 text-sm rounded hover:bg-accent",
                !prefs.source && "bg-accent",
              )}
              onClick={() => onUpdate({ source: "" })}
            >
              All Sources
            </button>
            {sources.map((s) => (
              <button
                key={s}
                className={cn(
                  "w-full text-left px-2 py-1.5 text-sm rounded hover:bg-accent",
                  prefs.source === s && "bg-accent",
                )}
                onClick={() => onUpdate({ source: s })}
              >
                {s}
              </button>
            ))}
          </PopoverContent>
        </Popover>
      )}

      {/* Tags filter */}
      {allTags.length > 0 && (
        <Popover>
          <PopoverTrigger asChild>
            <Button variant="outline" size="sm" className="h-8 gap-1.5">
              <Filter className="h-3.5 w-3.5" />
              Tags
              <Badge variant="secondary" className="h-5 px-1 text-[10px]">
                {prefs.tags.length}
              </Badge>
            </Button>
          </PopoverTrigger>
          <PopoverContent className="w-56 p-2" align="start">
            <div className="space-y-1 max-h-60 overflow-y-auto">
              {allTags.map((tag) => (
                <label
                  key={tag}
                  className="flex items-center gap-2 px-2 py-1 text-sm rounded hover:bg-accent cursor-pointer"
                >
                  <Checkbox
                    checked={prefs.tags.includes(tag)}
                    onCheckedChange={(c) => toggleTag(tag, !!c)}
                  />
                  <TagBadge tag={tag} />
                </label>
              ))}
            </div>
            <Separator className="my-2" />
            <div className="flex gap-2 px-2">
              <button
                className="text-xs text-muted-foreground hover:text-foreground"
                onClick={() => onUpdate({ tags: [...allTags] })}
              >
                Select all
              </button>
              <button
                className="text-xs text-muted-foreground hover:text-foreground"
                onClick={() => onUpdate({ tags: [] })}
              >
                Clear all
              </button>
            </div>
          </PopoverContent>
        </Popover>
      )}

      {/* Column visibility */}
      <Popover>
        <PopoverTrigger asChild>
          <Button variant="outline" size="sm" className="h-8 gap-1.5">
            <Columns3 className="h-3.5 w-3.5" />
            Columns
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-44 p-2" align="start">
          <div className="space-y-1">
            {allColumns.map((col) => (
              <label
                key={col}
                className="flex items-center gap-2 px-2 py-1 text-sm rounded hover:bg-accent cursor-pointer capitalize"
              >
                <Checkbox
                  checked={!prefs.hiddenColumns.includes(col)}
                  onCheckedChange={(c) => toggleColumn(col, !!c)}
                />
                {col}
              </label>
            ))}
          </div>
        </PopoverContent>
      </Popover>

      {/* Sort order */}
      <Popover>
        <PopoverTrigger asChild>
          <Button variant="outline" size="sm" className="h-8 gap-1.5">
            <ArrowUpDown className="h-3.5 w-3.5" />
            {prefs.sortOrder === "desc" ? "Newest" : "Oldest"}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-40 p-2" align="start">
          <button
            className={cn(
              "w-full text-left px-2 py-1.5 text-sm rounded hover:bg-accent",
              prefs.sortOrder === "desc" && "bg-accent",
            )}
            onClick={() => onUpdate({ sortOrder: "desc" })}
          >
            Newest First
          </button>
          <button
            className={cn(
              "w-full text-left px-2 py-1.5 text-sm rounded hover:bg-accent",
              prefs.sortOrder === "asc" && "bg-accent",
            )}
            onClick={() => onUpdate({ sortOrder: "asc" })}
          >
            Oldest First
          </button>
        </PopoverContent>
      </Popover>

      <Separator orientation="vertical" className="h-6" />

      {/* Live / Auto-scroll */}
      <div className="flex items-center gap-1.5">
        <span className={cn("h-2 w-2 rounded-full", statusColor)} />
        <label className="flex items-center gap-1.5 text-xs cursor-pointer">
          <Switch
            checked={prefs.liveStream}
            onCheckedChange={(c) => onUpdate({ liveStream: c })}
            className="scale-75"
          />
          Live
        </label>
      </div>

      <label className="flex items-center gap-1.5 text-xs cursor-pointer">
        <Switch
          checked={prefs.autoScroll}
          onCheckedChange={(c) => onUpdate({ autoScroll: c })}
          className="scale-75"
        />
        Auto-scroll
      </label>

      <Separator orientation="vertical" className="h-6" />

      {/* Export */}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline" size="sm" className="h-8 gap-1.5">
            <Download className="h-3.5 w-3.5" />
            Export
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem onClick={exportJSON}>Copy as JSON</DropdownMenuItem>
          <DropdownMenuItem onClick={exportTXT}>Copy as Text</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      {/* Refresh */}
      <Button variant="outline" size="icon" className="h-8 w-8" onClick={onRefresh} title="Refresh">
        <RefreshCw className="h-3.5 w-3.5" />
      </Button>

      {/* Clear */}
      <Button variant="outline" size="icon" className="h-8 w-8" onClick={handleClear} title="Clear all logs">
        <Trash2 className="h-3.5 w-3.5" />
      </Button>

      {/* Theme toggle */}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline" size="icon" className="h-8 w-8">
            {theme === "dark" ? (
              <Moon className="h-3.5 w-3.5" />
            ) : theme === "light" ? (
              <Sun className="h-3.5 w-3.5" />
            ) : (
              <Monitor className="h-3.5 w-3.5" />
            )}
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem onClick={() => setTheme("light")}>
            <Sun className="mr-2 h-4 w-4" />
            Light
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => setTheme("dark")}>
            <Moon className="mr-2 h-4 w-4" />
            Dark
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem onClick={() => setTheme("system")}>
            <Monitor className="mr-2 h-4 w-4" />
            System
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  )
}
