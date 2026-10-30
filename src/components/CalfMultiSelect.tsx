import { useId, useLayoutEffect, useRef, useState } from "react";
import { breeds, type Cow } from "../features/herd/types";

export function matchingCalves(cows: Cow[], selectedIds: number[], query: string, birthDate: string) {
  const term = query.trim().toLocaleLowerCase();
  const selected = new Set(selectedIds);
  return cows
    .filter(cow => cow.birth_date === birthDate && cow.ear_tag.toLocaleLowerCase().includes(term))
    .sort((a, b) => Number(selected.has(b.id ?? -1)) - Number(selected.has(a.id ?? -1)));
}

export function CalfMultiSelect({ cows, birthDate, selectedIds, onChange, placeholder = "Caută viței după crotalie" }: {
  cows: Cow[];
  birthDate: string;
  selectedIds: number[];
  onChange: (ids: number[]) => void;
  placeholder?: string;
}) {
  const id = useId();
  const input = useRef<HTMLInputElement>(null);
  const results = useRef<HTMLSpanElement>(null);
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const [position, setPosition] = useState({ top: 0, left: 0, width: 0, maxHeight: 220, transform: "none" });
  const matches = matchingCalves(cows, selectedIds, query, birthDate);
  const chosen = cows.filter(cow => cow.birth_date === birthDate && cow.id !== null && selectedIds.includes(cow.id));

  useLayoutEffect(() => {
    if (!open) return;
    const reposition = () => {
      const field = input.current?.getBoundingClientRect();
      if (!field) return;
      const dialog = input.current?.closest("dialog")?.getBoundingClientRect();
      const top = Math.max(8, dialog ? dialog.top + 8 : 8);
      const bottom = Math.min(window.innerHeight - 8, dialog ? dialog.bottom - 8 : window.innerHeight - 8);
      const below = Math.max(0, bottom - field.bottom - 4);
      const above = Math.max(0, field.top - top - 4);
      const upwards = below < 220 && above > below;
      setPosition({ left: field.left, width: field.width,
        top: upwards ? field.top - 4 : field.bottom + 4,
        maxHeight: Math.min(220, upwards ? above : below),
        transform: upwards ? "translateY(-100%)" : "none" });
    };
    reposition();
    window.addEventListener("resize", reposition);
    window.addEventListener("scroll", reposition, true);
    return () => { window.removeEventListener("resize", reposition); window.removeEventListener("scroll", reposition, true); };
  }, [open]);

  const toggle = (cow: Cow) => {
    if (cow.id === null) return;
    onChange(selectedIds.includes(cow.id) ? selectedIds.filter(id => id !== cow.id) : [...selectedIds, cow.id]);
    setQuery("");
    setActive(0);
    if (results.current) results.current.scrollTop = 0;
    input.current?.focus();
  };

  return <span className="calf-multi-select">
    {chosen.length > 0 && <span className="calf-multi-select-summary">{chosen.map(cow => cow.ear_tag).join(" · ")}</span>}
    <input ref={input} role="combobox" aria-autocomplete="list" aria-expanded={open}
      aria-controls={`${id}-list`} aria-activedescendant={open && matches[active] ? `${id}-${active}` : undefined}
      autoComplete="off" placeholder={placeholder} value={query}
      onFocus={() => { setOpen(true); setActive(0); }} onBlur={() => { setOpen(false); setQuery(""); }}
      onChange={event => {
        setQuery(event.target.value);
        setOpen(true);
        setActive(0);
        if (results.current) results.current.scrollTop = 0;
      }}
      onKeyDown={event => {
        if (event.key === "ArrowDown" || event.key === "ArrowUp") {
          event.preventDefault(); setOpen(true);
          setActive(index => Math.max(0, Math.min(matches.length - 1, index + (event.key === "ArrowDown" ? 1 : -1))));
        } else if (event.key === "Enter" && open) {
          event.preventDefault(); if (matches[active]) toggle(matches[active]);
        } else if (event.key === "Escape" && open) {
          event.preventDefault(); event.stopPropagation(); setOpen(false);
        }
      }} />
    {open && <span ref={results} style={position} className="cow-select-results calf-multi-select-results" role="listbox" aria-multiselectable="true" id={`${id}-list`} aria-label="Viței disponibili">
      {matches.map((cow, index) => <span role="option" id={`${id}-${index}`} key={cow.id}
        aria-selected={cow.id !== null && selectedIds.includes(cow.id)}
        onMouseDown={event => event.preventDefault()} onClick={() => toggle(cow)}>
        <span aria-hidden="true" className="calf-multi-check">{cow.id !== null && selectedIds.includes(cow.id) ? "✓" : ""}</span>
        {cow.ear_tag} · {breeds[cow.breed]}
      </span>)}
      {!matches.length && <span role="status">Niciun vițel găsit.</span>}
    </span>}
  </span>;
}
