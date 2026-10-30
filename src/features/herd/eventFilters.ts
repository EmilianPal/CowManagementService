import type { Birth, Cow, Insemination } from "./types";

export interface EventFilters {
  year: string;
  month: string;
  mother: string;
  related: string;
  breed: string;
  sex: string;
  association: string;
}
export const emptyEventFilters: EventFilters = {
  year: "", month: "", mother: "", related: "", breed: "", sex: "", association: "",
};
const contains = (value: string | undefined, query: string) =>
  !query.trim() || (value ?? "").toLowerCase().includes(query.trim().toLowerCase());
const inPeriod = (date: string, filter: EventFilters) =>
  (!filter.year || date.slice(0, 4) === filter.year) &&
  (!filter.month || date.slice(5, 7) === filter.month);

export interface BirthRow { birth: Birth; mother?: Cow; calf?: Cow }
export function birthRows(births: Birth[], cows: Cow[], filter: EventFilters): BirthRow[] {
  const byId = new Map(cows.map(c => [c.id, c]));
  const byBirth = new Map<number, Cow[]>();
  for (const cow of cows) {
    if (cow.birth_id !== null) byBirth.set(cow.birth_id, [...(byBirth.get(cow.birth_id) ?? []), cow]);
  }
  return [...births].sort((a, b) => b.date.localeCompare(a.date) || b.id - a.id).flatMap(birth => {
    const mother = byId.get(birth.mother_id);
    if (!inPeriod(birth.date, filter) || !contains(mother?.ear_tag, filter.mother)) return [];
    const calves = byBirth.get(birth.id) ?? [];
    if ((filter.association === "with" && !calves.length) || (filter.association === "without" && calves.length)) return [];
    const rows: BirthRow[] = calves.length
      ? [...calves].sort((a, b) => a.ear_tag.localeCompare(b.ear_tag)).map(calf => ({ birth, mother, calf }))
      : [{ birth, mother }];
    return rows.filter(({ calf }) => contains(calf?.ear_tag, filter.related) &&
      (!filter.breed || calf?.breed === filter.breed) && (!filter.sex || calf?.sex === filter.sex));
  });
}

export function inseminationRows(events: Insemination[], cows: Cow[], filter: EventFilters) {
  const byId = new Map(cows.map(c => [c.id, c]));
  return [...events].sort((a, b) => b.date.localeCompare(a.date) || b.id - a.id)
    .map(event => ({ event, dam: byId.get(event.dam_id), sire: byId.get(event.sire_id) }))
    .filter(({ event, dam, sire }) => inPeriod(event.date, filter) && contains(dam?.ear_tag, filter.mother) &&
      contains(sire?.ear_tag, filter.related) && (!filter.breed || dam?.breed === filter.breed) &&
      (!filter.association || (filter.association === "with" ? event.sire_id !== null : event.sire_id === null)));
}
