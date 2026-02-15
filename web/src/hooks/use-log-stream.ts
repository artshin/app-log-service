import { useEffect, useRef, useState } from "react"
import type { LogEntry } from "@/types/log-entry"

export type ConnectionStatus = "disconnected" | "connecting" | "connected"

export function useLogStream(
  enabled: boolean,
  onEntry: (entry: LogEntry) => void,
) {
  const [status, setStatus] = useState<ConnectionStatus>("disconnected")
  const onEntryRef = useRef(onEntry)
  onEntryRef.current = onEntry

  useEffect(() => {
    if (!enabled) {
      setStatus("disconnected")
      return
    }

    setStatus("connecting")
    const es = new EventSource("/stream")

    es.addEventListener("log", (event) => {
      try {
        onEntryRef.current(JSON.parse(event.data))
      } catch {
        // ignore parse errors
      }
    })

    es.onopen = () => setStatus("connected")

    es.onerror = () => {
      setStatus("connecting")
    }

    return () => {
      es.close()
      setStatus("disconnected")
    }
  }, [enabled])

  return status
}
