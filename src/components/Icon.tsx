import type { CSSProperties } from "react";
const paths: Record<string, string> = {
  herd: "M5 7 3 3m16 4 2-4M7 5h10l3 5-2 10H6L4 10Zm1 6h.01M16 11h.01M9 17h6M2 9l3 2m17-2-3 2",
  grid: "M3 3h7v7H3zM14 3h7v7h-7zM3 14h7v7H3zM14 14h7v7h-7z",
  leaf: "M20 3C9 2 2 8 5 15s15 6 15-12ZM4 21 15 10",
  heart:
    "M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1.1-1.1a5.5 5.5 0 0 0-7.8 7.8L12 21l8.8-8.6a5.5 5.5 0 0 0 0-7.8Z",
  calendar:
    "M8 2v4m8-4v4M3 10h18M5 4h14a2 2 0 0 1 2 2v14H3V6a2 2 0 0 1 2-2ZM7 14h2m4 0h4m-10 4h2",
  chart: "M4 3v18h17M9 16v-5m5 5V7m5 9V4",
  plus: "M12 5v14M5 12h14",
  search: "M21 21l-5-5M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0",
  filter: "M4 7h16M7 12h10m-7 5h4",
  download: "M12 3v12m-5-5 5 5 5-5M4 16v5h16v-5",
  chevron: "m9 5 7 7-7 7",
  close: "m6 6 12 12M6 18 18 6",
  undo: "M8 4 3 9l5 5M3 9h11a7 7 0 0 1 0 14",
  redo: "m16 4 5 5-5 5m5-5H10a7 7 0 0 0 0 14",
  edit: "m16 3 5 5-12 12-6 1 1-6ZM14 5l5 5",
  trash: "M3 6h18M9 6V3h6v3M5 6l1 15h12l1-15M10 10v7m4-7v7",
  logout: "M9 3H3v18h6m5-15 6 6-6 6m-7-6h13",
  check: "m5 12 4 4L19 6",
  refresh:
    "M20 7v5h-5M4 17v-5h5m-4-5a8 8 0 0 1 14-2l1 7M4 12l1 7a8 8 0 0 0 14-2",
  folder: "M3 6h7l2 3h9v12H3ZM3 6V3h7l2 3h8v3",
  arrow: "M5 12h14m-5-5 5 5-5 5",
};
export function Icon({
  name,
  size = 20,
  style,
}: {
  name: string;
  size?: number;
  style?: CSSProperties;
}) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.65"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      style={style}
    >
      <path d={paths[name] || paths.herd} />
    </svg>
  );
}
