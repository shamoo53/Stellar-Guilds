'use client'

import { useTheme } from 'next-themes'
import { useEffect, useState } from 'react'
import { Sun, Moon, Monitor } from 'lucide-react'

type Theme = 'light' | 'dark' | 'system'

const THEME_CYCLE: Theme[] = ['light', 'dark', 'system']

const icons: Record<Theme, React.ReactNode> = {
  light: <Sun size={18} />,
  dark: <Moon size={18} />,
  system: <Monitor size={18} />,
}

export function ThemeToggle() {
  const { theme, setTheme } = useTheme()
  const [mounted, setMounted] = useState(false)

  useEffect(() => setMounted(true), [])

  // Render neutral placeholder until hydrated to avoid mismatch
  if (!mounted) return <div className="w-9 h-9" />

  const current = (theme as Theme) ?? 'system'
  const next = THEME_CYCLE[(THEME_CYCLE.indexOf(current) + 1) % THEME_CYCLE.length]

  return (
    <button
      onClick={() => setTheme(next)}
      aria-label={`Current theme: ${current}. Switch to ${next}`}
      className="p-2 rounded-md hover:bg-stellar-lightNavy transition-colors"
    >
      {icons[current]}
    </button>
  )
}
