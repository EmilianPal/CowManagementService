import { useState } from "react";
import { Icon } from "../../components/Icon";
import { breeds, sexes, formatDate, today, type Birth, type Cow, type Insemination } from "./types";
import { birthRows, inseminationRows, emptyEventFilters, type EventFilters } from "./eventFilters";

interface Props {
  kind: "births" | "inseminations";
  births: Birth[];
  inseminations: Insemination[];
  cows: Cow[];
  writable: boolean;
  onSelect: (id: number) => void;
  onEditBirth: (birth: Birth) => void;
  onEditInsemination: (event: Insemination) => void;
  onDeleteBirth: (birth: Birth) => void;
  onDeleteInsemination: (event: Insemination) => void;
}
const months = ["Ianuarie", "Februarie", "Martie", "Aprilie", "Mai", "Iunie", "Iulie", "August", "Septembrie", "Octombrie", "Noiembrie", "Decembrie"];

export function ReproductionHistory(props: Props) {
  const { kind, births, inseminations, cows, writable, onSelect } = props;
  const isBirth = kind === "births";
  const [filters, setFilters] = useState({ births: { ...emptyEventFilters }, inseminations: { ...emptyEventFilters } });
  const filter = filters[kind];
  const update = (key: keyof EventFilters, value: string) => setFilters(current => ({ ...current, [kind]: { ...current[kind], [key]: value } }));
  const reset = () => setFilters(current => ({ ...current, [kind]: { ...emptyEventFilters } }));
  const years = [...new Set([today().slice(0, 4), ...(isBirth ? births : inseminations).map(e => e.date.slice(0, 4))])].sort().reverse();
  const birthsShown = isBirth ? birthRows(births, cows, filter) : [];
  const inseminationsShown = isBirth ? [] : inseminationRows(inseminations, cows, filter);
  const count = isBirth ? new Set(birthsShown.map(row => row.birth.id)).size : inseminationsShown.length;
  const calfCount = birthsShown.filter(row => row.calf).length;
  const total = (isBirth ? births : inseminations).length;
  const hasFilters = Object.values(filter).some(Boolean);
  const tag = (cow?: Cow) => cow?.id != null
    ? <button className="tag-button" onClick={() => onSelect(cow.id!)}>{cow.ear_tag}</button> : "—";
  const actions = (edit: () => void, remove: () => void, label: string) => writable && <div className="row-actions">
    <button className="icon-button" aria-label={`Editează ${label}`} onClick={edit}><Icon name="edit" size={17} /></button>
    <button className="icon-button danger-text" aria-label={`Șterge ${label}`} onClick={remove}><Icon name="trash" size={17} /></button>
  </div>;
  return <section className="panel reproduction-history">
    <div className="section-heading">
      <h2>{isBirth ? "Istoricul fătărilor" : "Istoricul montelor"}</h2>
      <span className="muted" role="status">{count} din {total} {isBirth ? "fătări" : "monte"}{isBirth && ` · ${calfCount} viței afișați`}</span>
    </div>
    <div className="advanced-filters">
      <div className="filter-grid">
        <label>Anul evenimentului<select value={filter.year} onChange={e => update("year", e.target.value)}>
          <option value="">Toți anii</option>{years.map(year => <option key={year}>{year}</option>)}
        </select></label>
        <label>Luna evenimentului<select value={filter.month} onChange={e => update("month", e.target.value)}>
          <option value="">Toate lunile</option>{months.map((month, index) => <option key={month} value={String(index + 1).padStart(2, "0")}>{month}</option>)}
        </select></label>
        <label>{isBirth ? "Crotalia mamei conține" : "Crotalia femelei conține"}<input value={filter.mother} onChange={e => update("mother", e.target.value)} placeholder="Orice fragment din crotalie" /></label>
        <label>{isBirth ? "Crotalia vițelului conține" : "Crotalia masculului conține"}<input value={filter.related} onChange={e => update("related", e.target.value)} placeholder="Orice fragment din crotalie" /></label>
        <label>{isBirth ? "Rasa vițelului" : "Rasa femelei"}<select value={filter.breed} onChange={e => update("breed", e.target.value)}>
          <option value="">Toate rasele</option>{Object.entries(breeds).map(([key, label]) => <option key={key} value={key}>{label}</option>)}
        </select></label>
        {isBirth && <label>Sexul vițelului<select value={filter.sex} onChange={e => update("sex", e.target.value)}>
          <option value="">Ambele sexe</option>{Object.entries(sexes).map(([key, label]) => <option key={key} value={key}>{label}</option>)}
        </select></label>}
        <label>{isBirth ? "Vițel asociat" : "Mascul asociat"}<select value={filter.association} onChange={e => update("association", e.target.value)}>
          <option value="">Toate evenimentele</option>
          <option value="with">{isBirth ? "Cu vițel asociat" : "Cu mascul asociat"}</option>
          <option value="without">{isBirth ? "Fără vițel asociat" : "Fără mascul specificat"}</option>
        </select></label>
      </div>
      <div className="filter-help"><p className="muted">Filtrele se aplică împreună, după data evenimentului. {isBirth ? "Fiecare vițel apare pe un rând separat." : "Luna fără an include aceeași lună din toți anii."}</p>
        <button className="text-button" onClick={reset} disabled={!hasFilters}>Resetează filtrele</button>
      </div>
    </div>
    <div className="table-scroll">
      <table>
        <caption className="sr-only">{isBirth ? "Fătări și datele de identificare ale vițeilor" : "Monte"} — rezultate filtrate</caption>
        <thead><tr>{(isBirth
          ? ["Nr.", "Data fătării", "Specie", "Rasa vițelului", "Sex", "Data nașterii", "Crotalia vițelului", "Crotalia mamei"]
          : ["Nr.", "Data montei", "Crotalia femelei", "Rasa femelei", "Crotalia masculului"]).map(label => <th scope="col" key={label}>{label}</th>)}
          {writable && <th scope="col">Acțiuni</th>}
        </tr></thead>
        <tbody>{isBirth ? birthsShown.map(({ birth, mother, calf }, index) => <tr key={`${birth.id}-${calf?.id ?? "none"}`}>
          <td>{index + 1}</td><td>{formatDate(birth.date)}</td><td>{calf ? "Bovine" : "—"}</td>
          <td>{calf ? breeds[calf.breed] : "—"}</td><td>{calf ? sexes[calf.sex] : "—"}</td>
          <td>{formatDate(calf?.birth_date ?? null)}</td><td>{calf ? tag(calf) : <span className="muted">Niciun vițel asociat</span>}</td><td>{tag(mother)}</td>
          {writable && <td>{actions(() => props.onEditBirth(birth), () => props.onDeleteBirth(birth), `fătarea din ${formatDate(birth.date)} a ${mother?.ear_tag ?? "mamei"}`)}</td>}
        </tr>) : inseminationsShown.map(({ event, dam, sire }, index) => <tr key={event.id}>
          <td>{index + 1}</td><td>{formatDate(event.date)}</td><td>{tag(dam)}</td><td>{dam ? breeds[dam.breed] : "—"}</td><td>{sire ? tag(sire) : "Nespecificat"}</td>
          {writable && <td>{actions(() => props.onEditInsemination(event), () => props.onDeleteInsemination(event), `monta din ${formatDate(event.date)} a ${dam?.ear_tag ?? "femelei"}`)}</td>}
        </tr>)}</tbody>
      </table>
    </div>
    {!count && <div className="empty"><Icon name={isBirth ? "leaf" : "heart"} size={34} />
      <h3>{total ? "Niciun eveniment nu corespunde filtrelor" : "Niciun eveniment înregistrat"}</h3>
      <p>{total ? "Schimbă criteriile sau resetează filtrele pentru a vedea toate evenimentele." : "Evenimentele adăugate vor apărea aici și în fișa bovinei."}</p>
    </div>}
  </section>;
}
