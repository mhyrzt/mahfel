export const themeCookieName = "mahfel_theme";

export const themes = ["system", "light", "dark"] as const;

export type Theme = (typeof themes)[number];

const themeValues: ReadonlySet<string> = new Set(themes);

export function isTheme(value: string | undefined): value is Theme {
  return value !== undefined && themeValues.has(value);
}
