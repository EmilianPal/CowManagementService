export const breeds = {
  BaltataRomaneasca: "Bălțată românească",
  Metis: "Metis",
  AmbardeenAngus: "Aberdeen Angus",
};
export const sexes = { Female: "Femelă", Male: "Mascul" };
export const categories = { Carne: "Carne", Lapte: "Lapte", Mixt: "Mixt" };
export interface Cow {
  id: number | null;
  farm_id: number;
  ear_tag: string;
  sex: keyof typeof sexes;
  breed: keyof typeof breeds;
  category: keyof typeof categories;
  birth_date: string;
  entry_date: string;
  exit_date: string | null;
  birth_id: number | null;
  birth_count: number;
  insemination_count: number;
}
export interface Birth {
  id: number;
  mother_id: number;
  date: string;
  farm_id: number;
}
export interface Insemination {
  id: number;
  dam_id: number;
  sire_id: number | null;
  date: string;
  farm_id: number;
}
export const today = () => {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
};
export const formatDate = (value: string | null) =>
  value
    ? value.split("-").reverse().join(".")
    : "—";
export const isPresent = (cow: Cow, date = today()) =>
  cow.entry_date <= date && (!cow.exit_date || cow.exit_date > date);
