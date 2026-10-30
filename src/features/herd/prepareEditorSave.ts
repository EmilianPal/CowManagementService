import { calfAssociationError } from "./reproduction";
import { today, type Birth, type Cow, type Insemination } from "./types";
import type { Editor } from "./editorTypes";

interface Context {
  editor: Editor;
  data: FormData;
  cows: Cow[];
  births: Birth[];
  inseminations: Insemination[];
  selectedCalfIds: number[];
  farmId: number;
}

type SavePlan =
  | { error: string }
  | { commandName: string; args: Record<string, unknown>; notice: string };

export function prepareEditorSave({ editor, data, cows, births, inseminations, selectedCalfIds, farmId }: Context): SavePlan {
  const str = (key: string) => String(data.get(key) || "").trim();
  const num = (key: string) => (str(key) ? Number(str(key)) : null);

  if (editor.kind === "cow") {
    const birthDate = str("birth_date");
    const entryDate = str("entry_date");
    const exitDate = str("exit_date");
    if (!str("ear_tag") || !birthDate || !entryDate) {
      return { error: "Completează crotalia și datele obligatorii." };
    }
    if (birthDate > today() || entryDate < birthDate || (exitDate && exitDate < entryDate)) {
      return { error: "Data nașterii nu poate fi în viitor, intrarea trebuie să fie după naștere, iar ieșirea după intrare." };
    }
    const birth = editor.birth ?? births.find(b => b.id === editor.value?.birth_id);
    if (editor.value && str("sex") !== editor.value.sex &&
      (births.some(b => b.mother_id === editor.value?.id) ||
       inseminations.some(i => i.dam_id === editor.value?.id || i.sire_id === editor.value?.id))) {
      return { error: "Nu poți schimba sexul unei bovine cu evenimente de reproducție. Corectează mai întâi evenimentele asociate." };
    }
    if (birth && birth.mother_id === editor.value?.id) {
      return { error: "Bovina nu poate fi propria mamă." };
    }
    const associationRemoved = Boolean(editor.value?.birth_id && birthDate !== editor.value.birth_date);
    const cow: Cow = {
      id: editor.value?.id ?? null,
      farm_id: farmId,
      ear_tag: str("ear_tag").toUpperCase(),
      sex: str("sex") as Cow["sex"],
      breed: str("breed") as Cow["breed"],
      category: str("category") as Cow["category"],
      birth_date: birthDate,
      entry_date: entryDate,
      exit_date: exitDate || null,
      birth_id: associationRemoved ? null : editor.birth?.id ?? editor.value?.birth_id ?? null,
      birth_count: editor.value?.birth_count ?? 0,
      insemination_count: editor.value?.insemination_count ?? 0,
    };
    const motherId = !editor.value && !editor.birth ? num("mother") : null;
    const mother = motherId === null ? undefined : cows.find(c => c.id === motherId);
    if (motherId !== null && !mother) {
      return { error: "Selectează o mamă din rezultate." };
    }
    if (mother && (mother.sex !== "Female" || birthDate < mother.birth_date)) {
      return { error: "Data nașterii vițelului trebuie să fie după data nașterii mamei." };
    }
    return mother
      ? { commandName: "add_cow_with_birth", args: { cow, motherId: mother.id }, notice: "Vițelul și fătarea au fost salvate." }
      : {
          commandName: editor.value ? "update_cow" : "add_cow",
          args: { cow },
          notice: associationRemoved
            ? "Datele bovinei au fost salvate. Asocierea cu fătarea anterioară a fost eliminată."
            : "Datele bovinei au fost salvate.",
        };
  }

  if (editor.kind === "birth" || editor.kind === "insemination") {
    const mother = cows.find(c => c.id === (editor.cow?.sex === "Female" ? editor.cow.id : num("mother")));
    const sire = cows.find(c => c.id === (editor.cow?.sex === "Male" ? editor.cow.id : num("sire")));
    const date = str("date");
    if (!mother || !date || date < mother.birth_date || date > today() || (sire && date < sire.birth_date)) {
      return { error: "Data evenimentului trebuie să fie după nașterea bovinelor și nu poate fi în viitor." };
    }
    const common = { id: editor.value?.id ?? null, farm_id: farmId, date };
    if (editor.kind === "birth") {
      for (const calfId of selectedCalfIds) {
        const calf = cows.find(c => c.id === calfId);
        const associationError = calf
          ? calfAssociationError(calf, mother.id, date, editor.value?.id)
          : "Vițelul selectat nu mai există în registru.";
        if (associationError) return { error: associationError };
      }
      return {
        commandName: "save_birth_with_calves",
        args: { birth: { ...common, mother_id: mother.id }, calfIds: selectedCalfIds },
        notice: "Evenimentul a fost salvat.",
      };
    }
    return {
      commandName: `${editor.value ? "update" : "add"}_insemination`,
      args: { insemination: { ...common, dam_id: mother.id, sire_id: sire?.id ?? null } },
      notice: "Evenimentul a fost salvat.",
    };
  }
  return { error: "Alege o înregistrare de editat." };
}
