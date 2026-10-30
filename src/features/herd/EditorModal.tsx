import { type Dispatch, type FormEventHandler, type SetStateAction } from "react";
import { Modal } from "../../components/Modal";
import { Icon } from "../../components/Icon";
import { CowSelect } from "../../components/CowSelect";
import { CalfMultiSelect } from "../../components/CalfMultiSelect";
import { DateInput } from "../../components/DateInput";
import { breeds, sexes, categories, today, formatDate, type Cow } from "./types";
import type { Editor } from "./editorTypes";

function Options({ values }: { values: Record<string, string> }) {
  return (
    <>
      {Object.entries(values).map(([value, label]) => (
        <option key={value} value={value}>
          {label}
        </option>
      ))}
    </>
  );
}

interface Props {
  editor: Editor;
  busy: boolean;
  modalError: string;
  cows: Cow[];
  eventDate: string;
  eventMotherId: number | null;
  selectedCalfIds: number[];
  setEventDate: Dispatch<SetStateAction<string>>;
  setEventMotherId: Dispatch<SetStateAction<number | null>>;
  setSelectedCalfIds: Dispatch<SetStateAction<number[]>>;
  setModalError: Dispatch<SetStateAction<string>>;
  tag: (id: number | null) => string;
  onClose: () => void;
  onDelete: () => void;
  onSave: FormEventHandler<HTMLFormElement>;
}

export function EditorModal({ editor, busy, modalError, cows, eventDate, eventMotherId, selectedCalfIds,
  setEventDate, setEventMotherId, setSelectedCalfIds, setModalError, tag, onClose, onDelete, onSave }: Props) {
  return (
    <Modal
      title={
        editor.kind === "cow"
          ? editor.value
            ? "Editează bovina"
            : editor.birth ? "Adaugă un vițel" : "Adaugă o bovină"
          : editor.kind === "birth"
            ? editor.value
              ? "Editează fătarea"
              : "Adaugă o fătare"
            : editor.kind === "insemination"
              ? editor.value
                ? "Editează monta"
                : "Adaugă o montă"
              : "Șterge înregistrarea"
      }
      subtitle={
        editor.kind === "delete"
          ? "Verifică înregistrarea înainte de a continua."
          : "O evidență completă începe cu date corecte."
      }
      busy={busy}
      onClose={onClose}
    >
      {modalError && (
        <div className="alert error" role="alert">
          {modalError}
        </div>
      )}
      {editor.kind === "delete" ? (
        <>
          <p className="delete-description">
            Sigur dorești să ștergi {editor.label}?{" "}
            {editor.command === "delete_cow" &&
              "Pentru femele se elimină montele și fătările asociate, iar vițeii rămân fără legătura cu fătarea. Pentru masculi se elimină doar asocierea lor din monte."}
            {editor.command === "delete_birth" &&
              "Vițeii asociați vor rămâne în registru, fără legătura cu această fătare."}
          </p>
          <div className="modal-actions">
            <button
              className="secondary"
              disabled={busy}
              onClick={onClose}
            >
              Renunță
            </button>
            <button
              className="danger-button solid"
              disabled={busy}
              onClick={onDelete}
            >
              {busy ? "Se șterge…" : "Șterge înregistrarea"}
            </button>
          </div>
        </>
      ) : (
        <form key={editor.kind} onSubmit={onSave} noValidate>
          <fieldset disabled={busy}>
            {editor.kind === "cow" && (
              <>
                {editor.birth && <p className="form-hint">Fătarea din {formatDate(editor.birth.date)} · Mama: {tag(editor.birth.mother_id)}. Fătarea este deja salvată; poți adăuga vițelul și mai târziu.</p>}
                <div className="form-grid">
                  <label className="span-two">
                    Crotalie <span>*</span>
                    <input
                      name="ear_tag"
                      required
                      maxLength={30}
                      defaultValue={editor.value?.ear_tag}
                      placeholder="De exemplu, RO500010635363"
                      autoFocus
                    />
                  </label>
                  <label>
                    Sex <span>*</span>
                    <select
                      name="sex"
                      defaultValue={editor.value?.sex || "Female"}
                    >
                      <Options values={sexes} />
                    </select>
                  </label>
                  <label>
                    Rasă <span>*</span>
                    <select
                      name="breed"
                      defaultValue={
                        editor.value?.breed || "BaltataRomaneasca"
                      }
                    >
                      <Options values={breeds} />
                    </select>
                  </label>
                  <label>
                    Categorie <span>*</span>
                    <select
                      name="category"
                      defaultValue={editor.value?.category || "Carne"}
                    >
                      <Options values={categories} />
                    </select>
                  </label>
                  <label>
                    Data nașterii <span>*</span>
                    {editor.birth ? <>
                      <input readOnly value={formatDate(editor.birth.date)} />
                      <input type="hidden" name="birth_date" value={editor.birth.date} />
                    </> : <DateInput name="birth_date" required max={today()} defaultValue={editor.value?.birth_date} />}

                  </label>
                  <label>
                    Data intrării <span>*</span>
                    <DateInput

                      name="entry_date"
                      required
                      defaultValue={editor.value?.entry_date || editor.birth?.date || today()}
                    />
                  </label>
                  <label>
                    Data ieșirii
                    <DateInput

                      name="exit_date"
                      defaultValue={editor.value?.exit_date || ""}
                    />
                  </label>
                  {!editor.value && !editor.birth && <label className="span-two">
                    Mama (opțional)
                    <CowSelect name="mother" cows={cows.filter(c => c.sex === "Female")}
                      placeholder="Caută mama după crotalie" />
                  </label>}
                </div>
              </>
            )}
            {(editor.kind === "birth" ||
              editor.kind === "insemination") && (
              <div className="form-grid">
                {editor.cow?.sex !== "Female" && <label className="span-two">
                  {editor.kind === "birth" ? "Mama" : "Femela"} <span>*</span>
                  <CowSelect name="mother" required cows={cows.filter(c => c.sex === "Female")}
                    onChange={cow => {
                      setEventMotherId(cow?.id ?? null);
                      if (editor.kind === "birth" && cow?.id != null) {
                        setSelectedCalfIds(ids => ids.filter(id => id !== cow.id));
                      }
                      setModalError("");
                    }}
                    defaultValue={editor.value ? "mother_id" in editor.value ? editor.value.mother_id : editor.value.dam_id : null} />
                </label>}
                {editor.kind === "insemination" && editor.cow?.sex !== "Male" && <label className="span-two">
                  Masculul (opțional)
                  <CowSelect name="sire" cows={cows.filter(c => c.sex === "Male")}
                    defaultValue={editor.value?.sire_id} placeholder="Caută un mascul sau lasă necompletat" />
                </label>}
                <label className="span-two">
                  Data evenimentului <span>*</span>
                  <DateInput
                    name="date"

                    required
                    max={today()}
                    value={eventDate}
                    onChange={event => {
                      const date = event.target.value;
                      setEventDate(date);
                      if (editor.kind === "birth") {
                        setSelectedCalfIds(ids => ids.filter(id => cows.some(c => c.id === id && c.birth_date === date)));
                      }
                      setModalError("");
                    }}
                  />
                </label>
                {editor.kind === "birth" && <div className="span-two">
                  <label>Viței asociați
                    <CalfMultiSelect
                      cows={cows.filter(c => c.id !== eventMotherId && (c.birth_id === null || c.birth_id === editor.value?.id))}
                      birthDate={eventDate}
                      selectedIds={selectedCalfIds}
                      onChange={ids => {
                        setSelectedCalfIds(ids);
                        setModalError("");
                      }} />
                  </label>
                  <p className="form-hint">Sunt afișați doar vițeii născuți la data fătării. Caută după crotalie și selectează unul sau mai mulți viței. Selectează din nou un vițel bifat pentru a-l elimina. Poți lăsa lista goală și asocia vițeii ulterior.</p>
                </div>}
              </div>
            )}
            <div className="modal-actions">
              <button
                type="button"
                className="secondary"
                onClick={onClose}
                disabled={busy}
              >
                Renunță
              </button>
              <button className="primary" disabled={busy}>
                {busy ? "Se salvează…" : "Salvează datele"}
                <Icon name="check" size={17} />
              </button>
            </div>
          </fieldset>
        </form>
      )}
    </Modal>
  );
}
