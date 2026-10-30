import type { Birth, Cow, Insemination } from "./types";

export type Editor =
  | { kind: "cow"; value?: Cow; birth?: Birth }
  | { kind: "birth"; value?: Birth; cow?: Cow }
  | { kind: "insemination"; value?: Insemination; cow?: Cow }
  | {
      kind: "delete";
      command: string;
      args: Record<string, unknown>;
      label: string;
    };
