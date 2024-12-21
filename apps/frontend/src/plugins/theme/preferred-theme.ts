import { type DarkTheme, isDarkTheme } from "./themes.ts";

export function usePreferredThemes() {
  // TODO: migrate theme preferences out of cosmetics to theme settings
  const cosmetics = useCosmetics();

  const dark = computed({
    get(): DarkTheme {
      const theme = cosmetics.value?.preferredDarkTheme;

      if (theme == null) {
        console.warn("[theme] cosmetics.preferredDarkTheme is not defined");
        return "dark";
      }

      if (!isDarkTheme(theme)) {
        console.warn(`[theme] cosmetics.preferredDarkTheme contains invalid value: ${theme}`);
        return "dark";
      }

      return theme;
    },
    set(value) {
      cosmetics.value.preferredDarkTheme = value;
    },
  });

  return reactive({
    dark,
    light: "light" as const,
  });
}
