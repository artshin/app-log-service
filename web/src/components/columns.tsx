import type { ColumnDef } from "@tanstack/react-table"
import type { LogEntry } from "@/types/log-entry"
import { LevelBadge } from "./level-badge"
import { TagBadge } from "./tag-badge"
import { formatTimeShort, getCallerDisplay } from "@/lib/utils"

export const columns: ColumnDef<LogEntry>[] = [
  {
    id: "level",
    accessorKey: "level",
    header: "Level",
    size: 100,
    minSize: 70,
    cell: ({ row }) => <LevelBadge level={row.original.level} />,
  },
  {
    id: "time",
    accessorKey: "timestamp",
    header: "Time",
    size: 80,
    minSize: 60,
    cell: ({ row }) => (
      <span className="font-mono text-xs text-muted-foreground whitespace-nowrap">
        {formatTimeShort(row.original.timestamp)}
      </span>
    ),
  },
  {
    id: "source",
    accessorKey: "source",
    header: "Source",
    size: 110,
    minSize: 60,
    cell: ({ row }) => (
      <span
        className="text-xs font-medium truncate block max-w-full"
        title={row.original.source}
      >
        {row.original.source}
      </span>
    ),
  },
  {
    id: "tags",
    accessorFn: (row) => row.tags.join(","),
    header: "Tags",
    size: 160,
    minSize: 80,
    cell: ({ row }) => {
      const tags = row.original.tags
      if (!tags.length)
        return <span className="text-xs text-muted-foreground">-</span>
      return (
        <div className="flex flex-wrap gap-1">
          {tags.map((t) => (
            <TagBadge key={t} tag={t} />
          ))}
        </div>
      )
    },
  },
  {
    id: "caller",
    accessorFn: (row) => getCallerDisplay(row.file, row.line),
    header: "Caller",
    size: 130,
    minSize: 60,
    cell: ({ row }) => {
      const display = getCallerDisplay(row.original.file, row.original.line)
      if (display === "-")
        return <span className="text-xs text-muted-foreground">-</span>
      return (
        <span
          className="text-xs font-mono text-muted-foreground truncate block max-w-full"
          title={`${row.original.file}:${row.original.line}`}
        >
          {display}
        </span>
      )
    },
  },
  {
    id: "message",
    accessorKey: "message",
    header: "Message",
    minSize: 200,
    cell: ({ row }) => (
      <span
        className="text-sm font-mono truncate block max-w-full"
        title={row.original.message}
      >
        {row.original.message}
      </span>
    ),
  },
]
