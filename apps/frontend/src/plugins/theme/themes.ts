export const LightThemes = ["light"] as const;

export type LightTheme = (typeof LightThemes)[number];

export const DarkThemes = ["dark", "oled", "retro"] as const;

export type DarkTheme = (typeof DarkThemes)[number];

export type Theme = LightTheme | DarkTheme;

export function isLightTheme(theme: Theme | (string & Record<never, never>)): theme is LightTheme {
  return LightThemes.includes(theme as any);
}

export function isDarkTheme(theme: Theme | (string & Record<never, never>)): theme is DarkTheme {
  return DarkThemes.includes(theme as any);
}

export type ThemeType = "light" | "dark";

export function getThemeType(
  theme: Theme | (string & Record<never, never>),
): ThemeType | "unknown" {
  if (isLightTheme(theme)) return "light";
  if (isDarkTheme(theme)) return "dark";
  return "unknown";
}
