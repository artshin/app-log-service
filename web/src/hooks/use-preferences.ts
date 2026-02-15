import { useCallback, useRef, useState } from "react"

const PREFS_KEY = "log-dashboard-prefs"

export interface Preferences {
  levels: string[]
  tags: string[]
  source: string
  sortOrder: "asc" | "desc"
  liveStream: boolean
  autoScroll: boolean
  hiddenColumns: string[]
  columnWidths: Record<string, number>
  searchQuery: string
}

const ALL_LEVELS = [
  "trace",
  "debug",
  "info",
  "notice",
  "warning",
  "error",
  "critical",
]

const DEFAULT_PREFS: Preferences = {
  levels: ALL_LEVELS,
  tags: [],
  source: "",
  sortOrder: "desc",
  liveStream: true,
  autoScroll: true,
  hiddenColumns: [],
  columnWidths: {},
  searchQuery: "",
}

function loadFromStorage(): Preferences {
  try {
    const raw = localStorage.getItem(PREFS_KEY)
    if (raw) {
      const parsed = JSON.parse(raw)
      return { ...DEFAULT_PREFS, ...parsed }
    }
  } catch {
    // ignore
  }
  return DEFAULT_PREFS
}

export function usePreferences() {
  const [prefs, setPrefsState] = useState<Preferences>(loadFromStorage)
  const prefsRef = useRef(prefs)
  prefsRef.current = prefs

  const update = useCallback((patch: Partial<Preferences>) => {
    setPrefsState((prev) => {
      const next = { ...prev, ...patch }
      try {
        localStorage.setItem(PREFS_KEY, JSON.stringify(next))
      } catch {
        // ignore
      }
      return next
    })
  }, [])

  return { prefs, update }
}
