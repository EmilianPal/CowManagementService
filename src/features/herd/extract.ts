import type { Cow } from "./types";

export const extractGroups = [
  {
    key: "adult",
    label: "Tauri, vaci și alte bovine și bubaline peste 2 ani",
    short: "Peste 2 ani",
  },
  {
    key: "young",
    label: "Bovine și bubaline între 6 luni și 2 ani",
    short: "Între 6 luni și 2 ani",
  },
  {
    key: "calf",
    label: "Bovine și bubaline de mai puțin de 6 luni",
    short: "Sub 6 luni",
  },
] as const;

export function isExtractDate(value: string) {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value) || value.startsWith("0000"))
    return false;
  const date = new Date(`${value}T00:00:00Z`);
  return (
    Number.isFinite(date.getTime()) && date.toISOString().slice(0, 10) === value
  );
}

// Calendar anniversaries, clamped to the last day for short months and leap years.
function anniversary(birth: string, months: number) {
  const [year, month, day] = birth.split("-").map(Number);
  const index = year * 12 + month - 1 + months;
  const targetYear = Math.floor(index / 12),
    targetMonth = index % 12;
  const lastDay = new Date(0);
  lastDay.setUTCFullYear(targetYear, targetMonth + 1, 0);
  return `${String(targetYear).padStart(4, "0")}-${String(targetMonth + 1).padStart(2, "0")}-${String(Math.min(day, lastDay.getUTCDate())).padStart(2, "0")}`;
}

export function buildExtract(cows: Cow[], date: string) {
  if (!isExtractDate(date))
    throw new Error("Selectează o dată validă pentru extras.");
  const groups = extractGroups.map((group) => ({
    ...group,
    males: [] as Cow[],
    females: [] as Cow[],
  }));
  for (const cow of cows) {
    if (
      cow.birth_date > date ||
      cow.entry_date > date ||
      (cow.exit_date && cow.exit_date <= date)
    )
      continue;
    const index =
      date > anniversary(cow.birth_date, 24)
        ? 0
        : date >= anniversary(cow.birth_date, 6)
          ? 1
          : 2;
    groups[index][cow.sex === "Male" ? "males" : "females"].push(cow);
  }
  for (const group of groups) {
    group.males.sort((a, b) => a.ear_tag.localeCompare(b.ear_tag));
    group.females.sort((a, b) => a.ear_tag.localeCompare(b.ear_tag));
  }
  const males = groups.reduce((sum, group) => sum + group.males.length, 0);
  const females = groups.reduce((sum, group) => sum + group.females.length, 0);
  return { date, groups, males, females, total: males + females };
}
