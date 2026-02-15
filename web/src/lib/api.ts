import type { LogEntry } from "@/types/log-entry"

export async function getLogs(): Promise<LogEntry[]> {
  const res = await fetch("/logs")
  if (!res.ok) throw new Error(`Failed to fetch logs: ${res.status}`)
  return res.json()
}

export async function clearLogs(): Promise<void> {
  const res = await fetch("/logs", { method: "DELETE" })
  if (!res.ok) throw new Error(`Failed to clear logs: ${res.status}`)
}
