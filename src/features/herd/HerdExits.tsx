import { useEffect, useMemo, useState } from "react";
import { command, romanianError } from "../../lib/api";
import { DateInput } from "../../components/DateInput";
import { FilterFields } from "./FilterFields";
import { emptyFilter, toCowFilter, validateFilters } from "./filters";
import { today, formatDate, breeds, sexes, type Cow } from "./types";
import { isExtractDate } from "./extract";

export function HerdExits({ writable, onSaved }: { writable: boolean; onSaved: () => Promise<void> }) {
  const [date, setDate] = useState(today);
  const [filter, setFilter] = useState(emptyFilter);
  const [search, setSearch] = useState("");
  const [cows, setCows] = useState<Cow[]>([]);
  const [selected, setSelected] = useState<Cow[]>([]);
  const selectedIds = new Set(selected.map(c => c.id));
  const displayedCows = [...selected, ...cows.filter(c => !selectedIds.has(c.id))];
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [revision, setRevision] = useState(0);
  const invalid = validateFilters(filter) || (!isExtractDate(date) || date > today() ? "Alege o dată validă, care nu este în viitor." : "");
  const query = useMemo(() => toCowFilter(filter, search, date, "present"), [filter, search, date]);
  useEffect(() => {
    let active = true;
    setCows([]); setLoading(true); setError("");
    if (invalid) { setLoading(false); return; }
    const timer = setTimeout(() => {
      command<Cow[]>("get_cows_filtered", { filter: query }).then(rows => {
        if (active) setCows(rows.filter(c => !c.exit_date && c.id !== null));
      }).catch(e => { if (active) setError(romanianError(e)); })
        .finally(() => { if (active) setLoading(false); });
    }, 150);
    return () => { active = false; clearTimeout(timer); };
  }, [query, invalid, revision]);

  async function save(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!event.currentTarget.checkValidity() || invalid || busy || loading || !selected.length) return;
    setBusy(true); setError(""); setNotice("");
    try {
      await command("exit_cows", { cowIds: selected.map(c => c.id), date });
      setNotice(`Ieșirea a fost înregistrată pentru ${selected.length} bovine, la ${formatDate(date)}.`);
      setSelected([]); setRevision(n => n + 1);
      await onSaved();
    } catch (e) { setError(romanianError(e)); }
    finally { setBusy(false); }
  }
  return <section className="panel">
    <form onSubmit={save} noValidate>
      <fieldset disabled={busy} className="exit-fields">
        <FilterFields value={filter} onChange={setFilter} date={date} onDateChange={setDate}
          search={search} onSearchChange={setSearch} status="present" onStatusChange={() => {}}
          hideStatus onReset={() => { setFilter(emptyFilter); setSearch(""); setDate(today()); }} />
        <p className="form-hint">Filtrele se aplică automat. Bovinele selectate rămân la începutul listei, chiar dacă nu corespund filtrelor. Ieșirea se înregistrează pentru întreaga selecție.</p>
        {(invalid || error) && <div className="alert error" role="alert">{invalid || error}</div>}
        {notice && <div className="alert" role="status">{notice}</div>}
        <label>Data ieșirii pentru bovinele selectate
          <DateInput value={date} onChange={e => setDate(e.target.value)} required max={today()} />
        </label>
        <label className="checkbox-label"><input type="checkbox" disabled={loading || !cows.length || !writable}
          checked={cows.length > 0 && cows.every(c => selectedIds.has(c.id))}
          onChange={e => {
            const checked = e.target.checked;
            setSelected(current => checked
              ? [...current, ...cows.filter(c => !current.some(s => s.id === c.id))]
              : current.filter(s => !cows.some(c => c.id === s.id)));
          }} />
          Selectează toate cele {cows.length} bovine din rezultate
        </label>
        <div className="table-scroll exit-table"><table>
          <thead><tr><th>Selectare</th><th>Crotalie</th><th>Rasă</th><th>Sex</th><th>Data nașterii</th><th>Data intrării</th></tr></thead>
          <tbody>{displayedCows.map(c => <tr key={c.id}>
            <td><input type="checkbox" aria-label={`Selectează ${c.ear_tag}`} disabled={!writable}
              checked={selectedIds.has(c.id)} onChange={e => {
                const checked = e.target.checked;
                setSelected(current => checked ? [...current, c] : current.filter(s => s.id !== c.id));
              }} /></td>
            <td>{c.ear_tag}</td><td>{breeds[c.breed]}</td><td>{sexes[c.sex]}</td><td>{formatDate(c.birth_date)}</td><td>{formatDate(c.entry_date)}</td>
          </tr>)}</tbody>
        </table></div>
        {loading ? <p role="status">Se încarcă bovinele…</p> : !cows.length && <p>Nu există bovine eligibile pentru filtrele alese.</p>}
        <p>{selected.length} bovine selectate · Data ieșirii: {formatDate(date)}</p>
        <div className="exit-actions">
        <button type="button" className="secondary" onClick={() => setRevision(n => n + 1)}>Reîncarcă lista</button>
        <button type="button" className="secondary" disabled={!selected.length} onClick={() => setSelected([])}>Golește selecția</button>
        <button className="primary" type="submit" disabled={!writable || busy || loading || !!invalid || !!error || !selected.length}>
          {busy ? "Se înregistrează…" : `Înregistrează ieșirea (${selected.length})`}
        </button>
        </div>
      </fieldset>
    </form>
  </section>;
}
