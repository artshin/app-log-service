import { useCallback, useEffect, useMemo } from "react"
import type { ColumnFiltersState, SortingState, VisibilityState } from "@tanstack/react-table"
import { useLogStore } from "@/hooks/use-log-store"
import { useLogStream } from "@/hooks/use-log-stream"
import { usePreferences } from "@/hooks/use-preferences"
import { getLogs, clearLogs } from "@/lib/api"
import { Toolbar } from "./toolbar"
import { StatsBar } from "./stats-bar"
import { LogDataTable } from "./log-data-table"
import { Toaster, toast } from "sonner"

export function Dashboard() {
  const store = useLogStore()
  const { prefs, update } = usePreferences()

  // Fetch initial logs
  useEffect(() => {
    getLogs()
      .then((entries) => {
        store.setEntries(entries)
        // Initialize tag preferences: if no saved tags, select all
        if (entries.length > 0) {
          const allTags = Array.from(
            new Set(entries.flatMap((e) => e.tags)),
          ).sort()
          if (prefs.tags.length === 0 && allTags.length > 0) {
            update({ tags: allTags })
          }
        }
      })
      .catch(() => toast.error("Failed to load logs"))
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // SSE stream
  const connectionStatus = useLogStream(prefs.liveStream, store.appendEntry)

  // Keep tag prefs in sync: auto-select new tags as they appear
  useEffect(() => {
    if (store.allTags.length > 0) {
      const newTags = store.allTags.filter((t) => !prefs.tags.includes(t))
      if (newTags.length > 0) {
        update({ tags: [...prefs.tags, ...newTags] })
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [store.allTags])

  const handleRefresh = useCallback(() => {
    getLogs()
      .then((entries) => store.setEntries(entries))
      .catch(() => toast.error("Failed to refresh logs"))
  }, [store])

  const handleClear = useCallback(() => {
    clearLogs()
      .then(() => {
        store.clearEntries()
        toast.success("Logs cleared")
      })
      .catch(() => toast.error("Failed to clear logs"))
  }, [store])

  // Derive table state from prefs
  const sorting: SortingState = useMemo(
    () => [{ id: "time", desc: prefs.sortOrder === "desc" }],
    [prefs.sortOrder],
  )

  const columnVisibility: VisibilityState = useMemo(() => {
    const vis: VisibilityState = {}
    for (const col of prefs.hiddenColumns) {
      vis[col] = false
    }
    return vis
  }, [prefs.hiddenColumns])

  // Build column filters from prefs
  const columnFilters: ColumnFiltersState = useMemo(() => {
    const filters: ColumnFiltersState = []
    return filters
  }, [])

  // Apply level + source + tag filtering via the data array
  const filteredData = useMemo(() => {
    let data = store.entries

    // Level filter
    if (prefs.levels.length < 7) {
      data = data.filter((e) => prefs.levels.includes(e.level.toLowerCase()))
    }

    // Source filter
    if (prefs.source) {
      data = data.filter((e) => e.source === prefs.source)
    }

    // Tag filter: hide entries that have any unchecked tag
    if (prefs.tags.length > 0 && store.allTags.length > 0) {
      const unchecked = store.allTags.filter((t) => !prefs.tags.includes(t))
      if (unchecked.length > 0) {
        data = data.filter(
          (e) =>
            e.tags.length === 0 ||
            !e.tags.some((t) => unchecked.includes(t)),
        )
      }
    }

    return data
  }, [store.entries, store.allTags, prefs.levels, prefs.source, prefs.tags])

  return (
    <div className="flex flex-col h-screen">
      <Toaster position="bottom-right" richColors />
      {/* Header */}
      <header className="flex items-center justify-between px-4 py-2 border-b shrink-0">
        <div className="flex items-center gap-3">
          <h1 className="text-lg font-semibold tracking-tight">Log Dashboard</h1>
          <a
            href="/info"
            className="text-xs text-muted-foreground hover:text-foreground transition"
          >
            API
          </a>
        </div>
      </header>

      {/* Toolbar */}
      <div className="px-4 py-2 border-b shrink-0">
        <Toolbar
          prefs={prefs}
          onUpdate={update}
          sources={store.sources}
          allTags={store.allTags}
          connectionStatus={connectionStatus}
          entries={filteredData}
          onRefresh={handleRefresh}
          onClear={handleClear}
        />
      </div>

      {/* Stats */}
      <div className="px-4 py-1.5 border-b shrink-0">
        <StatsBar
          totalCount={filteredData.length}
          levelCounts={store.levelCounts}
        />
      </div>

      {/* Table */}
      <LogDataTable
        data={filteredData}
        sorting={sorting}
        columnVisibility={columnVisibility}
        columnFilters={columnFilters}
        globalFilter={prefs.searchQuery}
        autoScroll={prefs.autoScroll}
      />
    </div>
  )
}
