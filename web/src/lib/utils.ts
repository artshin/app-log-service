import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/** FNV-1a hash — matches the Rust implementation in tags.rs */
export function fnv1aHash(str: string): number {
  let hash = 2166136261 >>> 0
  for (let i = 0; i < str.length; i++) {
    hash ^= str.charCodeAt(i)
    hash = Math.imul(hash, 16777619)
  }
  return hash >>> 0
}

/** Deterministic HSL color for a tag. Dark-mode aware. */
export function getTagColor(tag: string, dark: boolean) {
  const hash = fnv1aHash(tag)
  const hue = (hash * 137.508) % 360
  const saturation = 65 + (hash % 20) // 65-85%
  const bgL = dark ? 20 + (hash % 10) : 85 + (hash % 10) // light: 85-95%, dark: 20-30%
  const textL = dark ? 75 + (hash % 15) : 25 + (hash % 15) // light: 25-40%, dark: 75-90%
  return {
    bg: `hsl(${hue}, ${saturation}%, ${bgL}%)`,
    text: `hsl(${hue}, ${saturation}%, ${textL}%)`,
  }
}

/** Format ISO timestamp to "HH:MM:SS" */
export function formatTimeShort(iso: string): string {
  const d = new Date(iso)
  const pad = (n: number) => String(n).padStart(2, "0")
  return `${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())}:${pad(d.getUTCSeconds())}`
}

/** Format ISO timestamp to full "YYYY-MM-DD HH:MM:SS.mmm UTC" */
export function formatTimestamp(iso: string): string {
  const d = new Date(iso)
  const pad = (n: number, w = 2) => String(n).padStart(w, "0")
  return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())} ${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())}:${pad(d.getUTCSeconds())}.${pad(d.getUTCMilliseconds(), 3)} UTC`
}

/** Get caller display "filename:line" from full path */
export function getCallerDisplay(file: string, line: number): string {
  if (!file && !line) return "-"
  const filename = file ? file.split("/").pop() ?? "" : ""
  if (filename && line > 0) return `${filename}:${line}`
  if (filename) return filename
  return "-"
}
