import { useEffect, useRef, useState } from "react";
import { Icon } from "./Icon";

interface Props {
  value?: string;
  defaultValue?: string;
  onChange?: (event: { target: { value: string } }) => void;
  name?: string;
  required?: boolean;
  max?: string;
}

const display = (value: string) =>
  /^\d{4}-\d{2}-\d{2}$/.test(value)
    ? value.split("-").reverse().join(".")
    : value;

export function DateInput({ value, defaultValue = "", onChange, name, required, max }: Props) {
  const [iso, setIso] = useState(value ?? defaultValue);
  const [text, setText] = useState(display(value ?? defaultValue));
  const [error, setError] = useState("");
  const input = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (value !== undefined) {
      setIso(value);
      setText(display(value));
      setError("");
      input.current?.setCustomValidity("");
    }
  }, [value]);

  function update(text: string) {
    setText(text);
    const match = /^(\d{2})\.(\d{2})\.(\d{4})$/.exec(text);
    const next = match ? `${match[3]}-${match[2]}-${match[1]}` : "";
    const date = new Date(`${next}T12:00:00Z`);
    const valid = next && Number(match![3]) > 0 && !Number.isNaN(date.getTime()) && date.toISOString().slice(0, 10) === next;
    const message = text && (!valid || (max && next > max))
      ? `Introdu o dată validă în formatul zz.ll.aaaa${max ? `, cel târziu ${display(max)}` : ""}.`
      : "";
    setError(message);
    input.current?.setCustomValidity(message);
    if (!message) {
      setIso(next);
      onChange?.({ target: { value: next } });
    }
  }

  return (
    <span className="date-field">
      <span className="date-field-controls">
        <input ref={input} type="text" inputMode="numeric" placeholder="zz.ll.aaaa"
          value={text} required={required} aria-invalid={!!error}
          onChange={(event) => update(event.target.value)} />
        <input type="hidden" name={name} value={iso} />
        <span className="date-calendar" title="Alege data din calendar">
          <Icon name="calendar" size={24} style={{ color: "#287657" }} />
          <input type="date" aria-label="Alege data din calendar"
            value={iso} max={max}
            onChange={(event) => update(display(event.target.value))} />
        </span>
      </span>
      {error && <small className="date-field-error" role="alert">{error} Data introdusă nu a fost aplicată.</small>}
    </span>
  );
}
