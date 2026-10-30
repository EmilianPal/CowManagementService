import { useState } from "react";
import { Modal } from "../../components/Modal";
import { Icon } from "../../components/Icon";
import { CalfDetails, MotherDetails } from "./MotherDetails";
import { breeds, sexes, categories, today, formatDate, isPresent, type Cow, type Birth, type Insemination } from "./types";
import type { Editor } from "./editorTypes";

interface Props {
  cow: Cow;
  cows: Cow[];
  births: Birth[];
  inseminations: Insemination[];
  farmName: string;
  farmId: number;
  writable: boolean;
  onClose: () => void;
  onOpenEditor: (editor: Editor) => void;
}

export function CowDetailsModal({ cow: selectedCow, cows, births, inseminations, farmName, farmId, writable, onClose, onOpenEditor }: Props) {
  const [detailTab, setDetailTab] = useState("general");
  const selectedMother = cows.find(c => c.id === births.find(b => b.id === selectedCow.birth_id)?.mother_id);
  const tag = (id: number | null) => cows.find(c => c.id === id)?.ear_tag || "Nespecificat";
  return (
    <Modal
      title={selectedCow.ear_tag}
      subtitle="Fișa bovinei"
      onClose={onClose}
    >
      <div className="detail-tabs" role="tablist" aria-label="Secțiunile fișei">
        {[["general", "Date generale"], ["inseminations", "Monte"], ["births", "Fătări"]].map(([id, label], index, tabs) => (
          <button key={id} type="button" role="tab" id={`cow-tab-${id}`}
            aria-selected={detailTab === id} aria-controls={`cow-panel-${id}`}
            tabIndex={detailTab === id ? 0 : -1} onClick={() => setDetailTab(id)}
            onKeyDown={(event) => {
              const next = event.key === "ArrowRight" ? (index + 1) % 3 : event.key === "ArrowLeft" ? (index + 2) % 3 : event.key === "Home" ? 0 : event.key === "End" ? 2 : -1;
              if (next < 0) return;
              event.preventDefault();
              setDetailTab(tabs[next][0]);
              document.getElementById(`cow-tab-${tabs[next][0]}`)?.focus();
            }}>{label}</button>
        ))}
      </div>
      <section key={detailTab} className="detail-tab-panel" role="tabpanel" id={`cow-panel-${detailTab}`} aria-labelledby={`cow-tab-${detailTab}`} tabIndex={0}>
      {detailTab === "general" && <>
      <dl className="detail-grid">
        {[
          ["Rasă", breeds[selectedCow.breed]],
          ["Sex", sexes[selectedCow.sex]],
          ["Categorie", categories[selectedCow.category]],
          ["Stare", isPresent(selectedCow) ? "În fermă" : selectedCow.entry_date > today() ? "Intrare viitoare" : "Ieșită"],
          ["Data nașterii", formatDate(selectedCow.birth_date)],
          ["Data intrării", formatDate(selectedCow.entry_date)],
          ["Data ieșirii", formatDate(selectedCow.exit_date)],
          [
            "Mama",
            selectedMother?.ear_tag ?? "Neasociată",
          ],
          ["Fermă", farmName],
        ].map(([label, value]) => (
          <div key={label} className={label === "Mama" ? "mother-field" : undefined}>
            <dt>{label}</dt>
            <dd>{label === "Mama" && selectedMother
              ? <MotherDetails key={`${selectedCow.id}-${selectedMother.id}`} mother={selectedMother} farmName={farmName} />
              : value}</dd>
          </div>
        ))}
      </dl>
      </>}
      {detailTab !== "general" && <>
        <div className="detail-tab-heading">
          <h3>{detailTab === "births" ? "Fătări" : "Monte / însămânțări"}</h3>
          {writable && (detailTab !== "births" || selectedCow.sex === "Female") &&
            <button className="primary" onClick={() => onOpenEditor({ kind: detailTab === "births" ? "birth" : "insemination", cow: selectedCow })}>
              <Icon name="plus" size={17} />{detailTab === "births" ? "Adaugă o fătare" : "Adaugă o montă"}
            </button>}
        </div>
        <div className="detail-events">
          {(detailTab === "births"
            ? births.filter(b => b.mother_id === selectedCow.id).map(b => ({ id: b.id, date: b.date, text: "" }))
            : inseminations.filter(i => i.dam_id === selectedCow.id || i.sire_id === selectedCow.id).map(i => ({ id: i.id, date: i.date, text: `Femela: ${tag(i.dam_id)} · Masculul: ${tag(i.sire_id)}` })))
            .sort((a, b) => b.date.localeCompare(a.date)).map(event => (
              <div key={event.id}>
                <div className="detail-event-description">
                  {detailTab === "births" ? <>
                    Viței:
                    <div className="detail-calf-list">
                      {cows.some(c => c.birth_id === event.id)
                        ? cows.filter(c => c.birth_id === event.id).map(calf =>
                          <CalfDetails key={calf.id} calf={calf} farmName={farmName} />)
                        : "Niciun vițel asociat"}
                    </div>
                  </> : event.text}
                </div><strong>{formatDate(event.date)}</strong>
                {writable && <div className="detail-event-actions">
                  <button className="secondary" onClick={() => {
                    if (detailTab === "births") {
                      const value = births.find(b => b.id === event.id);
                      if (value) onOpenEditor({ kind: "birth", value, cow: selectedCow });
                    } else {
                      const value = inseminations.find(i => i.id === event.id);
                      if (value) onOpenEditor({ kind: "insemination", value, cow: selectedCow });
                    }
                  }}><Icon name="edit" size={16} /> Editează</button>
                  {detailTab === "births" && <button className="secondary" onClick={() => {
                    const birth = births.find(b => b.id === event.id);
                    if (birth) onOpenEditor({ kind: "cow", birth });
                  }}><Icon name="plus" size={16} /> Adaugă un vițel nou</button>}
                </div>}
              </div>
            ))}
          {(detailTab === "births" ? !births.some(b => b.mother_id === selectedCow.id) : !inseminations.some(i => i.dam_id === selectedCow.id || i.sire_id === selectedCow.id)) &&
            <p className="muted">{detailTab === "births" ? "Nu sunt înregistrate fătări." : "Nu sunt înregistrate monte."}</p>}
        </div>
      </>}
      </section>
      {writable && (
        <div className="modal-actions">
          <button
            className="danger-button"
            onClick={() =>
              onOpenEditor({
                kind: "delete",
                command: "delete_cow",
                args: {
                  cowId: selectedCow.id,
                  farmId: farmId,
                },
                label: `bovina ${selectedCow.ear_tag}`,
              })
            }
          >
            <Icon name="trash" size={17} />
            Șterge bovina
          </button>
          <button
            className="primary"
            onClick={() => onOpenEditor({ kind: "cow", value: selectedCow })}
          >
            <Icon name="edit" size={17} />
            Editează datele
          </button>
        </div>
      )}
    </Modal>
  );
}
