export type Page =
  "herd" | "overview" | "births" | "inseminations" | "reports" | "extract" | "exits";
export const navigation: { page: Page; label: string; icon: string }[] = [
  { page: "overview", label: "Privire de ansamblu", icon: "grid" },
  { page: "herd", label: "Registrul bovinelor", icon: "herd" },
  { page: "exits", label: "Ieșiri din fermă", icon: "logout" },
  { page: "extract", label: "Extras efectiv", icon: "chart" },
  { page: "inseminations", label: "Monte", icon: "heart" },
  { page: "births", label: "Fătări", icon: "calendar" },
  { page: "reports", label: "Rapoarte și export", icon: "chart" },
];
