import React, { useState, useEffect } from "react";
import { cn } from "@/lib/utils";
import { Search, X } from "lucide-react";

export interface SearchInputProps extends Omit<
  React.InputHTMLAttributes<HTMLInputElement>,
  "size"
> {
  onSearch?: (value: string) => void;
  debounceMs?: number;
  className?: string;
  placeholder?: string;
  showClearButton?: boolean;
}

const SearchInput = React.forwardRef<HTMLInputElement, SearchInputProps>(
  (
    {
      className,
      onSearch,
      debounceMs = 300,
      placeholder = "Search...",
      showClearButton = true,
      value: controlledValue,
      onChange,
      ...props
    },
    ref,
  ) => {
    const [internalValue, setInternalValue] = useState("");
    const isControlled = controlledValue !== undefined;
    const currentValue = (
      isControlled ? controlledValue : internalValue
    ) as string;

    useEffect(() => {
      console.log(`[SearchInput] Typing detected: "${currentValue || ""}"`);

      const timer = setTimeout(() => {
        console.log(
          `[SearchInput] Debounce completed (${debounceMs}ms) - Triggering search: "${currentValue || ""}"`,
        );
        onSearch?.(currentValue || "");
      }, debounceMs);

      return () => {
        console.log(
          `[SearchInput] Debounce timer cleared for: "${currentValue || ""}"`,
        );
        clearTimeout(timer);
      };
    }, [currentValue, debounceMs, onSearch]);

    const handleClear = () => {
      console.log("[SearchInput] Clear button clicked");
      if (!isControlled) {
        setInternalValue("");
      }
      onSearch?.("");
    };

    return (
      <div className={cn("relative w-full", className)}>
        <Search
          className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-stellar-slate"
          aria-hidden="true"
        />
        <input
          ref={ref}
          type="text"
          className={cn(
            "flex h-10 w-full rounded-md border border-stellar-slate bg-transparent pl-10 pr-10 py-2 text-sm placeholder:text-stellar-slate focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-gold-500 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50",
            className,
          )}
          placeholder={placeholder}
          value={currentValue}
          onChange={(e) => {
            console.log(`[SearchInput] Input changed: "${e.target.value}"`);
            if (!isControlled) {
              setInternalValue(e.target.value);
            }
            onChange?.(e);
          }}
          {...props}
        />
        {showClearButton && currentValue && (
          <button
            type="button"
            onClick={handleClear}
            className="absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2 text-stellar-slate hover:text-stellar-white transition-colors"
            aria-label="Clear search"
          >
            <X className="h-4 w-4" />
          </button>
        )}
      </div>
    );
  },
);

SearchInput.displayName = "SearchInput";

export { SearchInput };
