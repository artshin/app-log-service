import { ThemeProvider } from "@/components/theme-provider"
import { TooltipProvider } from "@/components/ui/tooltip"
import { Dashboard } from "@/components/dashboard"

export default function App() {
  return (
    <ThemeProvider>
      <TooltipProvider>
        <Dashboard />
      </TooltipProvider>
    </ThemeProvider>
  )
}
