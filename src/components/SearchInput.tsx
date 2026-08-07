import { Search, X } from 'lucide-react';

interface SearchInputProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}

export function SearchInput({ value, onChange, placeholder }: SearchInputProps) {
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
        className="search__input"
        type="search"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder ?? 'Search quests and games…'}
        autoComplete="off"
        spellCheck={false}
      />
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
