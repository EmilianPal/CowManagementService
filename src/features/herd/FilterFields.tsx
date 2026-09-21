import { DateInput } from "../../components/DateInput";
import { breeds, sexes, categories } from "./types";
import type { FilterValues, HerdStatus } from "./filters";

interface Props {
  value: FilterValues;
  onChange: (value: FilterValues) => void;
  date: string;
  onDateChange: (date: string) => void;
  search?: string;
  onSearchChange?: (search: string) => void;
  status: HerdStatus;
  onStatusChange: (status: HerdStatus) => void;
  onReset: () => void;
  hideStatus?: boolean;
}

export function FilterFields({
  value,
  onChange,
  date,
  onDateChange,
  search,
  onSearchChange,
  status,
  onStatusChange,
  onReset,
  hideStatus = false,
}: Props) {
  return (
    <div className="advanced-filters">
      <div className="filter-grid">
        {onSearchChange && (
          <label>
            Crotalia conține
            <input
              value={search}
              onChange={(e) => onSearchChange(e.target.value)}
              placeholder="Orice parte din crotalie"
            />
          </label>
        )}
        <label>
          Data de referință
          <DateInput
            
            value={date}
            onChange={(e) => onDateChange(e.target.value)}
          />
        </label>
        {!hideStatus && <label>
          Starea la data selectată
          <select
            value={status}
            onChange={(e) => onStatusChange(e.target.value as HerdStatus)}
          >
            <option value="all">Toate bovinele</option>
            <option value="present">În fermă</option>
            <option value="exited">Ieșite</option>
          </select>
        </label>}
        {(
          [
            ["breed", "Rasă", breeds, "Toate rasele"],
            ["sex", "Sex", sexes, "Ambele sexe"],
            ["category", "Categorie", categories, "Toate categoriile"],
          ] as const
        ).map(([key, label, options, all]) => (
          <label key={key}>
            {label}
            <select
              value={value[key]}
              onChange={(e) => onChange({ ...value, [key]: e.target.value })}
            >
              <option value="">{all}</option>
              {Object.entries(options).map(([key, text]) => (
                <option key={key} value={key}>
                  {text}
                </option>
              ))}
            </select>
          </label>
        ))}
        <label>
          Anul nașterii
          <input
            type="number"
            min="1"
            max="9999"
            step="1"
            value={value.born_in_year}
            onChange={(e) =>
              onChange({ ...value, born_in_year: e.target.value })
            }
            placeholder="Oricare"
          />
        </label>
        <label>
          Vârsta sub (luni)
          <input
            type="number"
            min="0"
            step="1"
            value={value.maximum_age_months}
            onChange={(e) =>
              onChange({ ...value, maximum_age_months: e.target.value })
            }
            placeholder="Fără limită"
          />
        </label>
        <label>
          Vârsta peste (luni)
          <input
            type="number"
            min="0"
            step="1"
            value={value.minimum_age_months}
            onChange={(e) =>
              onChange({ ...value, minimum_age_months: e.target.value })
            }
            placeholder="Fără limită"
          />
        </label>
        <label>
          Data intrării
          <DateInput
            
            value={value.entered_on}
            onChange={(e) => onChange({ ...value, entered_on: e.target.value })}
          />
        </label>
        <label>
          Data ieșirii
          <DateInput
            
            value={value.exited_on}
            onChange={(e) => onChange({ ...value, exited_on: e.target.value })}
          />
        </label>
      </div>
      <div className="filter-help">
        <p>
          Vârsta se calculează în luni împlinite, la data de referință.
          „Sub” exclude valoarea introdusă, iar „peste” o include (mai mare sau egală).
          Câmpurile goale nu restrâng rezultatele.
        </p>
        <button type="button" className="text-button" onClick={onReset}>
          Resetează filtrele
        </button>
      </div>
    </div>
  );
}
