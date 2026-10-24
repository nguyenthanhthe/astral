import { useEffect, useRef } from 'react';
import { Search, X } from 'lucide-react';

interface SearchInputProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}

export function SearchInput({ value, onChange, placeholder }: SearchInputProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  // "/" focuses the search from anywhere; Escape clears the query.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const typing =
        target &&
        (target.tagName === 'INPUT' ||
          target.tagName === 'TEXTAREA' ||
          target.tagName === 'SELECT' ||
          target.isContentEditable);
      if (e.key === '/' && !typing) {
        e.preventDefault();
        inputRef.current?.focus();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  return (
    <div className="search">
      <label htmlFor="quest-search" className="visually-hidden">
        Search quests
      </label>
      <span className="search__icon">
        <Search size={14} aria-hidden="true" />
      </span>
      <input
        id="quest-search"
        ref={inputRef}
        className="search__input"
        type="search"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Escape' && value.length > 0) {
            e.stopPropagation();
            onChange('');
          }
        }}
        placeholder={placeholder ?? 'Search quests and games…'}
        autoComplete="off"
        spellCheck={false}
      />
      {value.length === 0 && (
        <kbd className="search__kbd" aria-hidden="true">
          /
        </kbd>
      )}
      {value.length > 0 && (
        <button
          type="button"
          className="search__clear"
          onClick={() => onChange('')}
          aria-label="Clear search"
        >
          <X size={14} aria-hidden="true" />
        </button>
      )}
    </div>
  );
}
