import type { Theme } from "./themes.ts";

interface ThemeSettings {
  preference: Theme | "system";
  value: Theme;
}

export function useThemeSettings(getDefaultTheme?: () => Theme) {
  getDefaultTheme ??= () => "dark";

  const $settings = useCookie<ThemeSettings>("color-mode", {
    maxAge: 60 * 60 * 24 * 365 * 10,
    sameSite: "lax",
    secure: true,
    httpOnly: false,
    path: "/",
  });

  // reset theme settings to a default value if the cookie is missing or contains invalid value
  if ($settings.value == null || typeof $settings.value !== "object") {
    $settings.value = {
      preference: "system",
      value: getDefaultTheme(),
    };
  }

  return reactive({
    preferred: computed({
      get: () => $settings.value.preference ?? "system",
      set: (value) => ($settings.value.preference = value),
    }),

    active: computed({
      get: () => $settings.value.value ?? getDefaultTheme(),
      set: (value) => ($settings.value.value = value),
    }),
  });
}
