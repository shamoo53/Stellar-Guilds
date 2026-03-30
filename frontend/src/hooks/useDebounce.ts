import { useEffect, useState } from 'react'

/**
 * Custom hook for debouncing a value
 * @param value - The value to debounce
 * @param delay - The debounce delay in milliseconds (default: 300)
 * @returns The debounced value
 */
export function useDebounce<T>(value: T, delay: number = 300): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value)

  useEffect(() => {
    console.log(`[useDebounce] Value changed: "${value}" - Timer set for ${delay}ms`)

    const timer = setTimeout(() => {
      console.log(`[useDebounce] Timer completed - Value updated to: "${value}"`)
      setDebouncedValue(value)
    }, delay)

    return () => {
      console.log(`[useDebounce] Timer cleared for value: "${value}"`)
      clearTimeout(timer)
    }
  }, [value, delay])

  return debouncedValue
}
