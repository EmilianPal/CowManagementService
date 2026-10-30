import { useEffect, useId, useLayoutEffect, useRef, useState } from "react";
import { breeds, type Cow } from "../features/herd/types";

export function matchingCows(cows: Cow[], query: string) {
  return cows.filter(cow => cow.ear_tag.toLocaleLowerCase().includes(query.trim().toLocaleLowerCase()));
}

export function CowSelect({ cows, name, defaultValue, onChange, required = false, placeholder = "Caută după crotalie" }: {
  cows: Cow[]; name: string; defaultValue?: number | null; required?: boolean; placeholder?: string; onChange?: (cow: Cow | null) => void;
}) {
  const id = useId();
  const input = useRef<HTMLInputElement>(null);
  const results = useRef<HTMLSpanElement>(null);
  const [position, setPosition] = useState({ top: 0, left: 0, width: 0, maxHeight: 200, transform: "none" });
  const [selected, setSelected] = useState<number | null>(defaultValue ?? null);
  const [query, setQuery] = useState(cows.find(c => c.id === defaultValue)?.ear_tag ?? "");
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(0);
  const matches = matchingCows(cows, query);
  const selectionAvailable = selected === null || cows.some(cow => cow.id === selected);
  useEffect(() => {
    if (selectionAvailable) return;
    setSelected(null); setQuery(""); setActive(0);
    input.current?.setCustomValidity("");
    onChange?.(null);
  }, [selectionAvailable, onChange]);
  useLayoutEffect(() => {
    if (!open) return;
    const reposition = () => {
      if (!input.current) return;
      const field = input.current.getBoundingClientRect();
      const dialog = input.current.closest("dialog")?.getBoundingClientRect();
      const top = Math.max(8, dialog ? dialog.top + 8 : 8);
      const bottom = Math.min(window.innerHeight - 8, dialog ? dialog.bottom - 8 : window.innerHeight - 8);
      const below = Math.max(0, bottom - field.bottom - 4);
      const above = Math.max(0, field.top - top - 4);
      const upwards = below < 200 && above > below;
      const maxHeight = Math.min(200, upwards ? above : below);
      setPosition({ left: field.left, width: field.width,
        top: upwards ? field.top - 4 : field.bottom + 4, maxHeight,
        transform: upwards ? "translateY(-100%)" : "none" });
    };
    reposition();
    window.addEventListener("resize", reposition);
    window.addEventListener("scroll", reposition, true);
    return () => {
      window.removeEventListener("resize", reposition);
      window.removeEventListener("scroll", reposition, true);
    };
  }, [open]);
  useEffect(() => {
    const list = results.current;
    const option = document.getElementById(`${id}-${active}`);
    if (!open || !list || !option) return;
    // Scroll only the suggestions, never the surrounding dialog.
    const offset = option.offsetTop;
    if (offset < list.scrollTop) list.scrollTop = offset;
    else if (offset + option.offsetHeight > list.scrollTop + list.clientHeight) {
      list.scrollTop = offset + option.offsetHeight - list.clientHeight;
    }
  }, [open, active, id, query]);
  const choose = (cow: Cow) => {
    setSelected(cow.id); setQuery(cow.ear_tag); setOpen(false);
    input.current?.setCustomValidity("");
    onChange?.(cow);
  };
  return <span className="cow-select">
    <input ref={input} role="combobox" aria-autocomplete="list" aria-expanded={open}
      aria-controls={`${id}-list`} aria-activedescendant={open && matches[active] ? `${id}-${active}` : undefined}
      autoComplete="off" placeholder={placeholder} value={query} required={required}
      onFocus={e => { e.target.select(); setOpen(true); setActive(0); }} onBlur={() => setOpen(false)}
      onChange={e => {
        setQuery(e.target.value); setSelected(null); setOpen(true); setActive(0);
        onChange?.(null);
        e.target.setCustomValidity(e.target.value.trim() ? "Selectează o bovină din rezultate." : "");
      }}
      onKeyDown={e => {
        if (e.key === "ArrowDown" || e.key === "ArrowUp") {
          e.preventDefault(); setOpen(true);
          setActive(index => Math.max(0, Math.min(matches.length - 1, open ? index + (e.key === "ArrowDown" ? 1 : -1) : 0)));
        } else if (e.key === "Enter" && open) {
          e.preventDefault(); if (matches[active]) choose(matches[active]);
        } else if (e.key === "Escape" && open) { e.preventDefault(); e.stopPropagation(); setOpen(false); }
      }} />
    <input type="hidden" name={name} value={selectionAvailable ? selected ?? "" : ""} />
    {open && <span ref={results} style={position} className="cow-select-results" role="listbox" id={`${id}-list`} aria-label="Bovine găsite">
      {matches.map((cow, index) => <span role="option" id={`${id}-${index}`} key={cow.id}
        aria-selected={index === active} onMouseDown={e => e.preventDefault()} onClick={() => choose(cow)}>
        {cow.ear_tag} · {breeds[cow.breed]}
      </span>)}
      {!matches.length && <span role="status">Nicio bovină găsită.</span>}
    </span>}
  </span>;
}
