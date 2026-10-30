import { Icon } from "../../components/Icon";
import { breeds, formatDate, type Cow, type Birth, type Insemination } from "./types";

interface Props {
  present: Cow[];
  births: Birth[];
  inseminations: Insemination[];
  tag: (id: number | null) => string;
}

export function OverviewPage({ present, births, inseminations, tag }: Props) {
  return (
    <div className="overview-grid">
      <section className="panel">
        <div className="section-heading">
          <h2>Structura efectivului</h2>
          <span className="muted">Bovine prezente</span>
        </div>
        <div className="breed-bars">
          {Object.entries(breeds).map(([key, label]) => {
            const count = present.filter((c) => c.breed === key).length;
            return (
              <div key={key}>
                <div>
                  <span>{label}</span>
                  <strong>{count} bovine</strong>
                </div>
                <div className="bar-track">
                  <span
                    style={{
                      width: `${present.length ? (count / present.length) * 100 : 0}%`,
                    }}
                  />
                </div>
              </div>
            );
          })}
        </div>
      </section>
      <section className="panel">
        <div className="section-heading">
          <h2>Evenimente recente</h2>
          <Icon name="calendar" />
        </div>
        <div className="event-list">
          {[
            ...births.map((b) => ({
              key: `b${b.id}`,
              title: "Fătare înregistrată",
              cow: tag(b.mother_id),
              date: b.date,
            })),
            ...inseminations.map((i) => ({
              key: `i${i.id}`,
              title: "Montă înregistrată",
              cow: tag(i.dam_id),
              date: i.date,
            })),
          ]
            .sort((a, b) => b.date.localeCompare(a.date))
            .slice(0, 6)
            .map((item) => (
              <div className="event" key={item.key}>
                <span className="stat-icon green">
                  <Icon name="calendar" size={18} />
                </span>
                <div>
                  <strong>{item.title}</strong>
                  <small>{item.cow}</small>
                </div>
                <span>{formatDate(item.date)}</span>
              </div>
            ))}
          {!births.length && !inseminations.length && (
            <p className="muted">
              Evenimentele fermei vor apărea aici.
            </p>
          )}
        </div>
      </section>
    </div>
  );
}
