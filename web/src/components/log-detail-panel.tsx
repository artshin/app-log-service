import type { LogEntry } from "@/types/log-entry"
import { formatTimestamp } from "@/lib/utils"
import { TagBadge } from "./tag-badge"

export function LogDetailPanel({ entry }: { entry: LogEntry }) {
  return (
    <div className="space-y-3 px-4 py-4">
      <div>
        <span className="text-muted-foreground text-xs uppercase tracking-wide block mb-1">
          Full Message
        </span>
        <pre className="font-mono text-sm bg-muted px-3 py-2 rounded border whitespace-pre-wrap break-words">
          {entry.message}
        </pre>
      </div>
      <div className="grid grid-cols-[100px_1fr] gap-y-2 gap-x-4 text-sm">
        <Field label="ID" value={entry.id} />
        <Field label="Timestamp" value={formatTimestamp(entry.timestamp)} />
        {entry.file && <Field label="File" value={entry.file} />}
        {entry.function && <Field label="Function" value={entry.function} />}
        {entry.line > 0 && <Field label="Line" value={String(entry.line)} />}
        {entry.deviceId && <Field label="Device ID" value={entry.deviceId} />}
        {entry.userId && <Field label="User ID" value={entry.userId} />}
        {entry.tags.length > 0 && (
          <>
            <span className="text-muted-foreground text-xs uppercase tracking-wide">
              Tags
            </span>
            <div className="flex flex-wrap gap-1">
              {entry.tags.map((t) => (
                <TagBadge key={t} tag={t} />
              ))}
            </div>
          </>
        )}
        {entry.metadata && Object.keys(entry.metadata).length > 0 && (
          <>
            <span className="text-muted-foreground text-xs uppercase tracking-wide">
              Metadata
            </span>
            <pre className="font-mono text-xs bg-muted px-2 py-1 rounded border overflow-x-auto">
              {Object.entries(entry.metadata)
                .map(([k, v]) => `${k}: ${v}`)
                .join("\n")}
            </pre>
          </>
        )}
      </div>
    </div>
  )
}

function Field({ label, value }: { label: string; value: string }) {
  return (
    <>
      <span className="text-muted-foreground text-xs uppercase tracking-wide">
        {label}
      </span>
      <code className="font-mono text-xs bg-muted px-2 py-1 rounded border break-all">
        {value}
      </code>
    </>
  )
}
