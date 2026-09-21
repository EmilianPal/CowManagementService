import { DateInput } from "../../components/DateInput";
import { useEffect, useState } from "react";
import { command, romanianError } from "../../lib/api";
import { Icon } from "../../components/Icon";
import { buildExtract, isExtractDate } from "./extract";
import { today, formatDate, type Cow } from "./types";

export function HerdExtract() {
  const [date, setDate] = useState(today);
  const [revision, setRevision] = useState(0);
  const [result, setResult] = useState<ReturnType<typeof buildExtract> | null>(
    null,
  );
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  const valid = isExtractDate(date);
  useEffect(() => {
    let active = true;
    setResult(null);
    setError("");
    if (!isExtractDate(date)) {
      setLoading(false);
      return;
    }
    setLoading(true);
    command<Cow[]>("get_cows_in_the_plantation", { date })
      .then((cows) => {
        if (active) setResult(buildExtract(cows, date));
      })
      .catch((error) => {
        if (active) setError(romanianError(error));
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [date, revision]);
  const extract = result?.date === date ? result : null;

  return (
    <section className="herd-extract" aria-label="Extras efectiv">
      <div className="panel extract-summary">
        <div className="extract-controls">
          <label>
            Data extrasului
            <DateInput
              
              value={date}
              onChange={(e) => {
                setResult(null);
                setDate(e.target.value);
              }}
            />
          </label>
          <button
            className="secondary"
            disabled={loading || !valid}
            onClick={() => {
              setResult(null);
              setRevision((value) => value + 1);
            }}
          >
            <Icon name="refresh" size={17} />
            Reîncarcă
          </button>
        </div>
        <p className="extract-description">
          Întregul efectiv prezent în fermă la data aleasă, indiferent de
          filtrele din registru.
        </p>
        {!valid && (
          <div className="alert error" role="alert">
            Selectează o dată validă pentru extras.
          </div>
        )}
        {error && (
          <div className="alert error" role="alert">
            {error}
          </div>
        )}
        {valid && loading && (
          <p className="extract-loading" role="status">
            Se calculează efectivul…
          </p>
        )}
        {extract && (
          <>
            <div className="table-scroll">
              <table className="extract-table">
                <caption>Efectiv la {formatDate(extract.date)}</caption>
                <thead>
                  <tr>
                    <th scope="col">Grupa de vârstă</th>
                    <th scope="col">Masculi</th>
                    <th scope="col">Femele</th>
                    <th scope="col">Total</th>
                  </tr>
                </thead>
                <tbody>
                  {extract.groups.map((group) => (
                    <tr key={group.key}>
                      <th scope="row">{group.label}</th>
                      <td>{group.males.length}</td>
                      <td>{group.females.length}</td>
                      <td>{group.males.length + group.females.length}</td>
                    </tr>
                  ))}
                </tbody>
                <tfoot>
                  <tr>
                    <th scope="row">Total efectiv</th>
                    <td>{extract.males}</td>
                    <td>{extract.females}</td>
                    <td>{extract.total}</td>
                  </tr>
                </tfoot>
              </table>
            </div>
            {extract.total === 0 && (
              <p className="extract-description">
                Nu sunt înregistrate bovine prezente la această dată.
              </p>
            )}
          </>
        )}
        <p className="extract-note">
          La exact 6 luni și la exact 2 ani, bovina intră în grupa „Între 6 luni
          și 2 ani”. Bovinele ieșite în ziua selectată nu sunt incluse.
        </p>
      </div>
      {extract && (
        <details className="extract-details" key={extract.date}>
          <summary>
            Vezi crotaliile pe grupe <span>{extract.total} bovine</span>
          </summary>
          <div className="extract-lists">
            {extract.groups.flatMap((group) =>
              [
                { label: "Masculi", cows: group.males, key: "males" },
                { label: "Femele", cows: group.females, key: "females" },
              ].map(({ label, cows, key }) => (
                <section className="panel" key={`${group.key}-${key}`}>
                  <div className="section-heading">
                    <h2>
                      {label} · {group.short.toLocaleLowerCase("ro")}
                    </h2>
                    <span>{cows.length}</span>
                  </div>
                  <div className="extract-list-scroll">
                    <table aria-label={`${label} · ${group.short}`}>
                      <thead>
                        <tr>
                          <th scope="col">Nr.</th>
                          <th scope="col">Crotalie</th>
                        </tr>
                      </thead>
                      <tbody>
                        {cows.map((cow, index) => (
                          <tr key={cow.id}>
                            <td>{index + 1}</td>
                            <td>{cow.ear_tag}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                    {!cows.length && (
                      <p className="extract-description">
                        Nicio bovină în această grupă.
                      </p>
                    )}
                  </div>
                </section>
              )),
            )}
          </div>
        </details>
      )}
    </section>
  );
}
