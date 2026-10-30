import type { Dispatch, SetStateAction } from "react";
import { Icon } from "../../components/Icon";
import { FilterFields } from "./FilterFields";
import type { FilterValues, HerdStatus } from "./filters";

interface Props {
  filter: FilterValues;
  setFilter: Dispatch<SetStateAction<FilterValues>>;
  reportDate: string;
  setReportDate: Dispatch<SetStateAction<string>>;
  search: string;
  setSearch: Dispatch<SetStateAction<string>>;
  status: HerdStatus;
  setStatus: Dispatch<SetStateAction<HerdStatus>>;
  resetFilters: () => void;
  filterError: string;
  filterLoadError: string;
  busy: boolean;
  onExport: () => void;
}

export function ReportsPage({ filter, setFilter, reportDate, setReportDate, search, setSearch,
  status, setStatus, resetFilters, filterError, filterLoadError, busy, onExport }: Props) {
  return (
    <section className="panel report-panel">
      <div className="report-icon">
        <Icon name="chart" size={32} />
      </div>
      <h2>Situația efectivului</h2>
      <p>
        Exportă un registru Excel cu crotalii, rase, sex, date și
        numărul de monte și fătări.
      </p>
      <FilterFields
        value={filter}
        onChange={setFilter}
        date={reportDate}
        onDateChange={setReportDate}
        search={search}
        onSearchChange={setSearch}
        status={status}
        onStatusChange={setStatus}
        onReset={resetFilters}
      />
      <label className="checkbox-label">
        <input
          type="checkbox"
          checked={status === "present"}
          onChange={(event) =>
            setStatus(event.target.checked ? "present" : "all")
          }
        />
        Doar bovinele prezente în fermă la data de referință
      </label>
      <p className="form-hint">
        Debifează pentru a include și bovinele ieșite din fermă.
        Celelalte filtre rămân aplicate.
      </p>
      {(filterError || filterLoadError) && (
        <div className="alert error" role="alert">
          {filterError || filterLoadError}
        </div>
      )}
      <div className="report-summary">
        <Icon name="folder" />
        <span>
          Raportul folosește criteriile de mai sus și include toate
          înregistrările corespunzătoare.
        </span>
      </div>
      <button
        className="primary"
        disabled={!!filterError || busy}
        onClick={onExport}
      >
        <Icon name="download" size={18} />
        Exportă în Excel
      </button>
    </section>
  );
}
