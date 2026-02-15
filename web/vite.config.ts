import path from "path"
import { defineConfig } from "vite"
import react from "@vitejs/plugin-react"
import tailwindcss from "@tailwindcss/vite"

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    proxy: {
      "/logs": "http://localhost:9006",
      "/stream": "http://localhost:9006",
      "/info": "http://localhost:9006",
    },
  },
  build: {
    outDir: "../server/static/app",
    emptyOutDir: true,
  },
})
