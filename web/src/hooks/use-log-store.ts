import { useCallback, useReducer } from "react"
import type { LogEntry } from "@/types/log-entry"

const MAX_ENTRIES = 10_000

export interface LogState {
  entries: LogEntry[]
  levelCounts: Record<string, number>
  sources: string[]
  allTags: string[]
}

type Action =
  | { type: "SET"; entries: LogEntry[] }
  | { type: "APPEND"; entry: LogEntry }
  | { type: "CLEAR" }

function deriveStats(entries: LogEntry[]) {
  const levelCounts: Record<string, number> = {}
  const sourceSet = new Set<string>()
  const tagSet = new Set<string>()
  for (const e of entries) {
    levelCounts[e.level] = (levelCounts[e.level] ?? 0) + 1
    sourceSet.add(e.source)
    for (const t of e.tags) tagSet.add(t)
  }
  return {
    levelCounts,
    sources: Array.from(sourceSet).sort(),
    allTags: Array.from(tagSet).sort(),
  }
}

function reducer(state: LogState, action: Action): LogState {
  switch (action.type) {
    case "SET": {
      const entries = action.entries.slice(-MAX_ENTRIES)
      return { entries, ...deriveStats(entries) }
    }
    case "APPEND": {
      const entries =
        state.entries.length >= MAX_ENTRIES
          ? [...state.entries.slice(1), action.entry]
          : [...state.entries, action.entry]
      const e = action.entry
      const levelCounts = {
        ...state.levelCounts,
        [e.level]: (state.levelCounts[e.level] ?? 0) + 1,
      }
      const sources = state.sources.includes(e.source)
        ? state.sources
        : [...state.sources, e.source].sort()
      let allTags = state.allTags
      const newTags = e.tags.filter((t) => !allTags.includes(t))
      if (newTags.length > 0) {
        allTags = [...allTags, ...newTags].sort()
      }
      return { entries, levelCounts, sources, allTags }
    }
    case "CLEAR":
      return { entries: [], levelCounts: {}, sources: [], allTags: [] }
  }
}

const INITIAL: LogState = {
  entries: [],
  levelCounts: {},
  sources: [],
  allTags: [],
}

export function useLogStore() {
  const [state, dispatch] = useReducer(reducer, INITIAL)

  const setEntries = useCallback(
    (entries: LogEntry[]) => dispatch({ type: "SET", entries }),
    [],
  )
  const appendEntry = useCallback(
    (entry: LogEntry) => dispatch({ type: "APPEND", entry }),
    [],
  )
  const clearEntries = useCallback(() => dispatch({ type: "CLEAR" }), [])

  return { ...state, setEntries, appendEntry, clearEntries }
}
