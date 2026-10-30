import { useEffect, useId, useLayoutEffect, useRef, useState } from "react";
import { breeds, categories, sexes, formatDate, isPresent, today, type Cow } from "./types";

function CowHoverDetails({ animal, farmName, relation, activation }: { animal: Cow; farmName: string; relation: "mamei" | "vițelului"; activation: "hover" | "press" }) {
  const id = useId();
  const trigger = useRef<HTMLButtonElement>(null);
  const card = useRef<HTMLElement>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const [open, setOpen] = useState(false);
  const cancelClose = () => { clearTimeout(timer.current); };
  const show = () => { cancelClose(); setOpen(true); };
  const scheduleClose = () => {
    cancelClose();
    timer.current = setTimeout(() => {
      if (!trigger.current?.matches(":focus-visible") && !card.current?.contains(document.activeElement)) setOpen(false);
    }, 150);
  };
  const blur = (next: EventTarget | null) => {
    if (!(next instanceof Node) || (!trigger.current?.contains(next) && !card.current?.contains(next))) {
      cancelClose(); setOpen(false);
    }
  };
  useEffect(() => () => clearTimeout(timer.current), []);
  useLayoutEffect(() => {
    const panel = card.current;
    const anchor = trigger.current;
    if (!open || !panel || !anchor) return;
    // The popover sits above the dialog and is not clipped by its scrolling panels.
    if (typeof panel.showPopover === "function") panel.showPopover();
    const reposition = () => {
      const rect = anchor.getBoundingClientRect();
      const width = Math.min(320, window.innerWidth - 24);
      const below = Math.max(0, window.innerHeight - rect.bottom - 20);
      const above = Math.max(0, rect.top - 20);
      const upwards = below < 200 && above > below;
      panel.style.width = `${width}px`;
      panel.style.maxHeight = `${Math.min(250, upwards ? above : below)}px`;
      const height = panel.getBoundingClientRect().height;
      panel.style.left = `${Math.max(12, Math.min(rect.left, window.innerWidth - width - 12))}px`;
      panel.style.top = `${upwards ? rect.top - height - 8 : rect.bottom + 8}px`;
    };
    reposition();
    const escape = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault(); event.stopPropagation();
      cancelClose(); setOpen(false);
    };
    window.addEventListener("resize", reposition);
    window.addEventListener("scroll", reposition, true);
    document.addEventListener("keydown", escape, true);
    return () => {
      window.removeEventListener("resize", reposition);
      window.removeEventListener("scroll", reposition, true);
      document.removeEventListener("keydown", escape, true);
      if (panel.matches(":popover-open")) panel.hidePopover();
    };
  }, [open]);
  return <div className="mother-details">
    <button ref={trigger} type="button" className="mother-trigger"
      aria-label={`Arată datele ${relation} ${animal.ear_tag}`} aria-expanded={open} aria-controls={id}
      onPointerEnter={activation === "hover" ? show : cancelClose} onPointerLeave={scheduleClose}
      onFocus={activation === "hover" ? show : undefined}
      onBlur={event => blur(event.relatedTarget)}
      onClick={activation === "press" ? () => { cancelClose(); setOpen(value => !value); } : show}>{animal.ear_tag}</button>
    <section ref={card} id={id} popover="manual" hidden={!open} className="mother-card" tabIndex={0}
      aria-label={`Datele ${relation} ${animal.ear_tag}`} onPointerEnter={cancelClose} onPointerLeave={scheduleClose}
      onFocus={cancelClose} onBlur={event => blur(event.relatedTarget)}>
      <dl>
        {[
          ["Rasă", breeds[animal.breed]],
          ["Sex", sexes[animal.sex]],
          ["Categorie", categories[animal.category]],
          ["Stare", isPresent(animal) ? "În fermă" : animal.entry_date > today() ? "Intrare viitoare" : "Ieșită"],
          ["Data nașterii", formatDate(animal.birth_date)],
          ["Data intrării", formatDate(animal.entry_date)],
          ["Data ieșirii", formatDate(animal.exit_date)],
          ["Fermă", farmName],
          ["Monte", animal.insemination_count],
          ["Fătări", animal.birth_count],
        ].map(([label, value]) => <div key={label}><dt>{label}</dt><dd>{value}</dd></div>)}
      </dl>
    </section>
  </div>;
}

export function MotherDetails({ mother, farmName }: { mother: Cow; farmName: string }) {
  return <CowHoverDetails animal={mother} farmName={farmName} relation="mamei" activation="press" />;
}

export function CalfDetails({ calf, farmName }: { calf: Cow; farmName: string }) {
  return <CowHoverDetails animal={calf} farmName={farmName} relation="vițelului" activation="press" />;
}
