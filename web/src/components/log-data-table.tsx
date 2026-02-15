import { Fragment, memo, useCallback, useEffect, useRef } from "react"
import {
  flexRender,
  getCoreRowModel,
  getExpandedRowModel,
  getFilteredRowModel,
  getSortedRowModel,
  useReactTable,
  type ColumnFiltersState,
  type ExpandedState,
  type SortingState,
  type VisibilityState,
} from "@tanstack/react-table"
import type { LogEntry } from "@/types/log-entry"
import { columns } from "./columns"
import { LogDetailPanel } from "./log-detail-panel"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { cn } from "@/lib/utils"
import { Copy } from "lucide-react"
import { toast } from "sonner"

interface LogDataTableProps {
  data: LogEntry[]
  sorting: SortingState
  columnVisibility: VisibilityState
  columnFilters: ColumnFiltersState
  globalFilter: string
  autoScroll: boolean
}

const MemoRow = memo(function MemoRow({
  entry,
  cells,
  isExpanded,
  onToggle,
  onCopy,
}: {
  entry: LogEntry
  cells: ReturnType<
    ReturnType<typeof useReactTable<LogEntry>>["getRowModel"]
  >["rows"][0]["getVisibleCells"]
  isExpanded: boolean
  onToggle: () => void
  onCopy: (e: React.MouseEvent) => void
}) {
  const visibleCells = cells()
  return (
    <Fragment>
      <TableRow
        className="cursor-pointer group"
        onClick={onToggle}
        data-state={isExpanded ? "selected" : undefined}
      >
        {visibleCells.map((cell) => (
          <TableCell
            key={cell.id}
            style={{ width: cell.column.getSize() }}
          >
            {flexRender(cell.column.columnDef.cell, cell.getContext())}
          </TableCell>
        ))}
        <TableCell className="w-10 p-0">
          <button
            className="opacity-0 group-hover:opacity-100 p-1.5 text-muted-foreground hover:text-foreground transition"
            onClick={onCopy}
            title="Copy JSON"
          >
            <Copy className="h-3.5 w-3.5" />
          </button>
        </TableCell>
      </TableRow>
      {isExpanded && (
        <TableRow>
          <TableCell colSpan={visibleCells.length + 1} className="p-0 bg-muted/30">
            <LogDetailPanel entry={entry} />
          </TableCell>
        </TableRow>
      )}
    </Fragment>
  )
})

export function LogDataTable({
  data,
  sorting,
  columnVisibility,
  columnFilters,
  globalFilter,
  autoScroll,
}: LogDataTableProps) {
  const wrapperRef = useRef<HTMLDivElement>(null)
  const prevLenRef = useRef(data.length)

  const table = useReactTable({
    data,
    columns,
    state: { sorting, columnVisibility, columnFilters, globalFilter, expanded: {} as ExpandedState },
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getExpandedRowModel: getExpandedRowModel(),
    columnResizeMode: "onChange",
    getRowId: (row) => row.id,
    enableGlobalFilter: true,
    globalFilterFn: (row, _columnId, filterValue) => {
      const search = (filterValue as string).toLowerCase()
      const entry = row.original
      return (
        entry.message.toLowerCase().includes(search) ||
        entry.source.toLowerCase().includes(search) ||
        entry.tags.some((t) => t.toLowerCase().includes(search))
      )
    },
  })

  // Auto-scroll to top when new entries arrive
  useEffect(() => {
    if (autoScroll && data.length > prevLenRef.current && wrapperRef.current) {
      wrapperRef.current.scrollTop = 0
    }
    prevLenRef.current = data.length
  }, [data.length, autoScroll])

  const handleCopy = useCallback((entry: LogEntry, e: React.MouseEvent) => {
    e.stopPropagation()
    navigator.clipboard
      .writeText(JSON.stringify(entry, null, 2))
      .then(() => toast.success("Copied to clipboard"))
      .catch(() => toast.error("Failed to copy"))
  }, [])

  const rows = table.getRowModel().rows

  return (
    <div
      ref={wrapperRef}
      className="flex-1 overflow-auto border rounded-md"
    >
      <Table>
        <TableHeader className="sticky top-0 z-10 bg-background">
          {table.getHeaderGroups().map((hg) => (
            <TableRow key={hg.id}>
              {hg.headers.map((header) => (
                <TableHead
                  key={header.id}
                  style={{ width: header.getSize() }}
                  className="relative select-none"
                >
                  {header.isPlaceholder
                    ? null
                    : flexRender(header.column.columnDef.header, header.getContext())}
                  {header.column.getCanResize() && (
                    <div
                      onMouseDown={header.getResizeHandler()}
                      onTouchStart={header.getResizeHandler()}
                      className={cn(
                        "absolute right-0 top-1 bottom-1 w-1 cursor-col-resize rounded bg-border hover:bg-primary transition-colors",
                        header.column.getIsResizing() && "bg-primary",
                      )}
                    />
                  )}
                </TableHead>
              ))}
              <TableHead className="w-10" />
            </TableRow>
          ))}
        </TableHeader>
        <TableBody>
          {rows.length === 0 ? (
            <TableRow>
              <TableCell
                colSpan={columns.length + 1}
                className="h-32 text-center text-muted-foreground"
              >
                {data.length === 0
                  ? "No log entries yet. Send a log to get started."
                  : "No entries match the current filters."}
              </TableCell>
            </TableRow>
          ) : (
            rows.map((row) => (
              <MemoRow
                key={row.id}
                entry={row.original}
                cells={() => row.getVisibleCells()}
                isExpanded={row.getIsExpanded()}
                onToggle={() => row.toggleExpanded()}
                onCopy={(e) => handleCopy(row.original, e)}
              />
            ))
          )}
        </TableBody>
      </Table>
    </div>
  )
}
