export const emptyFilter = {
  breed: "",
  sex: "",
  category: "",
  born_in_year: "",
  minimum_age_months: "",
  maximum_age_months: "",
  entered_on: "",
  exited_on: "",
};
export type FilterValues = typeof emptyFilter;
export type HerdStatus = "all" | "present" | "exited";

export function validateFilters(filter: FilterValues): string {
  for (const key of [
    "born_in_year",
    "minimum_age_months",
    "maximum_age_months",
  ] as const) {
    if (
      filter[key] &&
      (!/^\d+$/.test(filter[key]) || !Number.isSafeInteger(Number(filter[key])))
    )
      return "Introdu numere întregi pozitive sau zero pentru vârstă și un an valid.";
  }
  if (
    filter.born_in_year &&
    (Number(filter.born_in_year) < 1 || Number(filter.born_in_year) > 9999)
  )
    return "Anul nașterii trebuie să fie cuprins între 1 și 9999.";
  if (
    filter.minimum_age_months &&
    filter.maximum_age_months &&
    Number(filter.minimum_age_months) >= Number(filter.maximum_age_months)
  )
    return "Valoarea pentru „Vârsta peste” trebuie să fie mai mică decât cea pentru „Vârsta sub”.";
  return "";
}

export function toCowFilter(
  filter: FilterValues,
  search: string,
  date: string,
  status: HerdStatus,
) {
  return {
    date,
    ear_tag_contains: search.trim() || null,
    breed: filter.breed || null,
    sex: filter.sex || null,
    category: filter.category || null,
    born_in_year: filter.born_in_year ? Number(filter.born_in_year) : null,
    minimum_age_months: filter.minimum_age_months
      ? Number(filter.minimum_age_months)
      : null,
    maximum_age_months: filter.maximum_age_months
      ? Number(filter.maximum_age_months)
      : null,
    entered_on: filter.entered_on || null,
    exited_on: filter.exited_on || null,
    show_only_entered: status === "present",
    show_only_exited: status === "exited",
  };
}
