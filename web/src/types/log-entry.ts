export interface LogEntry {
  id: string
  timestamp: string
  level: string
  message: string
  userId?: string
  deviceId: string
  source: string
  metadata: Record<string, string>
  tags: string[]
  file: string
  function: string
  line: number
}
