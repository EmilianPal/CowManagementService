import type { Dispatch, SetStateAction } from "react";
import { Icon } from "../../components/Icon";
import { FilterFields } from "./FilterFields";
import { breeds, sexes, formatDate, isPresent, type Cow } from "./types";
import type { FilterValues, HerdStatus } from "./filters";
import type { Editor } from "./editorTypes";

export type SortOrder = "tag" | "newest" | "oldest";

interface Props {
  cows: Cow[];
  visible: Cow[];
  selected: number | null;
  setSelected: Dispatch<SetStateAction<number | null>>;
  writable: boolean;
  busy: boolean;
  loading: boolean;
  filtering: boolean;
  history: { undo: boolean; redo: boolean };
  status: HerdStatus;
  setStatus: Dispatch<SetStateAction<HerdStatus>>;
  reportDate: string;
  setReportDate: Dispatch<SetStateAction<string>>;
  filter: FilterValues;
  setFilter: Dispatch<SetStateAction<FilterValues>>;
  search: string;
  setSearch: Dispatch<SetStateAction<string>>;
  showFilters: boolean;
  setShowFilters: Dispatch<SetStateAction<boolean>>;
  sort: SortOrder;
  setSort: Dispatch<SetStateAction<SortOrder>>;
  filterError: string;
  filterLoadError: string;
  resetFilters: () => void;
  openEditor: (editor: Editor) => void;
  onUndo: () => void;
  onRedo: () => void;
  onRefresh: () => void;
}

export function RegistryPage({ cows, visible, selected, setSelected, writable, busy, loading, filtering,
  history, status, setStatus, reportDate, setReportDate, filter, setFilter, search, setSearch,
  showFilters, setShowFilters, sort, setSort, filterError, filterLoadError, resetFilters,
  openEditor, onUndo, onRedo, onRefresh }: Props) {
  return (
    <section className="panel registry">
      <div className="panel-top">
        <div className="tabs">
          {(
            [
              {
                key: "all",
                label: "Toate bovinele",
                count: cows.length,
              },
              {
                key: "present",
                label: "În fermă",
                count: cows.filter((c) => isPresent(c, reportDate))
                  .length,
              },
              {
                key: "exited",
                label: "Ieșite",
                count: cows.filter(
                  (c) => c.exit_date && c.exit_date <= reportDate,
                ).length,
              },
            ] as const
          ).map((t) => (
            <button
              key={t.key}
              className={status === t.key ? "active" : ""}
              onClick={() => setStatus(t.key)}
            >
              {t.label}
              <span>{t.count}</span>
            </button>
          ))}
        </div>
        <div className="history-controls">
          <button
            title="Anulează ultima modificare"
            aria-label="Anulează ultima modificare"
            disabled={busy || !history.undo || !writable}
            onClick={onUndo}
          >
            <Icon name="undo" size={17} />
          </button>
          <button
            title="Refă modificarea"
            aria-label="Refă modificarea"
            disabled={busy || !history.redo || !writable}
            onClick={onRedo}
          >
            <Icon name="redo" size={17} />
          </button>
        </div>
      </div>
      <div className="toolbar">
        <div className="search">
          <Icon name="search" size={18} />
          <input
            aria-label="Caută după crotalie"
            placeholder="Caută după crotalie…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
          {search && (
            <button
              aria-label="Șterge căutarea"
              onClick={() => setSearch("")}
            >
              <Icon name="close" size={15} />
            </button>
          )}
        </div>
        <button
          className={`secondary ${showFilters ? "selected" : ""}`}
          aria-expanded={showFilters}
          onClick={() => setShowFilters(!showFilters)}
        >
          <Icon name="filter" size={18} />
          Filtre
          {Object.values(filter).filter(Boolean).length > 0 && (
            <span className="filter-count">
              {Object.values(filter).filter(Boolean).length}
            </span>
          )}
        </button>
        <select
          aria-label="Sortează bovinele"
          value={sort}
          onChange={(e) => setSort(e.target.value as typeof sort)}
        >
          <option value="tag">Crotalie: crescător</option>
          <option value="newest">Cele mai tinere</option>
          <option value="oldest">Cele mai în vârstă</option>
        </select>
        <button
          className="icon-button"
          title="Reîncarcă"
          aria-label="Reîncarcă"
          disabled={loading || busy}
          onClick={onRefresh}
        >
          <Icon name="refresh" size={18} />
        </button>
      </div>
      {showFilters && (
        <FilterFields
          value={filter}
          onChange={setFilter}
          date={reportDate}
          onDateChange={setReportDate}
          status={status}
          onStatusChange={setStatus}
          onReset={resetFilters}
        />
      )}
      {(filterError || filterLoadError) && (
        <div className="alert error" role="alert">
          {filterError || filterLoadError}
        </div>
      )}
      <p className="filter-context">
        Data de referință: {formatDate(reportDate || null)} ·{" "}
        {filtering ? "Se filtrează…" : `${visible.length} bovine`}
      </p>
      <div className="table-scroll">
        <table>
          <thead>
            <tr>
              <th className="row-number">Nr.</th>
              <th>Crotalie</th>
              <th>Rasă</th>
              <th>Sex</th>
              <th>Data nașterii</th>
              <th>Data intrării</th>
              <th>Data ieșirii</th>
              <th>Categorie</th>
              <th>Monte</th>
              <th>Fătări</th>
              <th>Stare</th>
              <th>
                <span className="sr-only">Detalii</span>
              </th>
            </tr>
          </thead>
          <tbody>
            {visible.map((c, index) => (
              <tr
                key={c.id}
                className={selected === c.id ? "selected-row" : ""}
              >
                <td className="row-number">
                  {index + 1}
                </td>
                <td>
                  <button
                    className="tag-button"
                    onClick={() => setSelected(c.id)}
                  >
                    <span className="mini-cow">
                      <Icon name="herd" size={17} />
                    </span>
                    {c.ear_tag}
                  </button>
                </td>
                <td>{breeds[c.breed]}</td>
                <td>
                  <span
                    className={`sex-badge ${c.sex === "Female" ? "female" : "male"}`}
                  >
                    {c.sex === "Female" ? "♀" : "♂"}
                  </span>
                  {sexes[c.sex]}
                </td>
                <td>{formatDate(c.birth_date)}</td>
                <td>{formatDate(c.entry_date)}</td>
                <td className={!c.exit_date ? "muted" : ""}>
                  {formatDate(c.exit_date)}
                </td>
                <td>
                  <span className="category-badge">{c.category}</span>
                </td>
                <td className="numeric">{c.insemination_count}</td>
                <td className="numeric">{c.birth_count}</td>
                <td>
                  <span
                    className={`state-badge ${isPresent(c, reportDate) ? "present" : "exited"}`}
                  >
                    <i />
                    {isPresent(c, reportDate)
                      ? "În fermă"
                      : c.entry_date > reportDate
                        ? "Intrare viitoare"
                        : "Ieșită"}
                  </span>
                </td>
                <td>
                  <button
                    className="icon-button"
                    aria-label={`Fișa bovinei ${c.ear_tag}`}
                    onClick={() => setSelected(c.id)}
                  >
                    <Icon name="chevron" size={16} />
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {!visible.length && (
          <div className="empty">
            <span className="empty-icon">
              <Icon
                name={loading || filtering ? "refresh" : "herd"}
                size={32}
              />
            </span>
            <h3>
              {loading || filtering
                ? "Se încarcă registrul…"
                : cows.length
                  ? "Nicio bovină nu corespunde căutării"
                  : "Registrul tău începe aici"}
            </h3>
            <p>
              {cows.length
                ? "Încearcă altă crotalie sau modifică filtrele."
                : "Adaugă prima bovină pentru a construi evidența fermei."}
            </p>
            {!cows.length && writable && !loading && !filtering && (
              <button
                className="secondary"
                onClick={() => openEditor({ kind: "cow" })}
              >
                <Icon name="plus" size={17} />
                Adaugă prima bovină
              </button>
            )}
          </div>
        )}
      </div>
      <div className="table-footer">
        <span>{visible.length} bovine</span>
      </div>
    </section>
  );
}
