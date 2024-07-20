import { ref, computed } from "vue";

export type Theme = "system" | "light" | "dark" | "oled" | "retro";

export const DarkThemes: Theme[] = ["dark", "oled", "retro"];

export type SystemTheme = "unknown" | "light" | "dark";

function useNativeTheme() {
  if (import.meta.server) {
    let clientHint;

    switch (useRequestHeader("Sec-CH-Prefers-Color-Scheme")) {
      case "light":
        clientHint = "light";
        break;
      case "dark":
        clientHint = "dark";
        break;
      default:
        clientHint = "unknown";
    }

    return computed(() => clientHint as SystemTheme);
  }

  const lightPreference = window.matchMedia("(prefers-color-scheme: light)");

  const isLight = ref(lightPreference.matches);

  const onPreferenceChange = ({ matches }: MediaQueryListEvent) => (isLight.value = matches);

  lightPreference.addEventListener("change", onPreferenceChange);

  onScopeDispose(() => lightPreference.removeEventListener("change", onPreferenceChange));

  return computed<SystemTheme>(() => (isLight.value ? "light" : "dark"));
}

interface ThemeSettings {
  preference: Theme;
  value: Exclude<Theme, "system">;
}

export default defineNuxtPlugin((nuxtApp) => {
  const $nativeTheme = useNativeTheme();

  const $themeSettings = useCookie<ThemeSettings>("color-mode", {
    maxAge: 60 * 60 * 24 * 365 * 10,
    sameSite: "lax",
    secure: true,
    httpOnly: false,
    path: "/",
    default: () => ({
      preference: "system",
      // if we have a client hint - use the associated theme, otherwise use dark
      // theme, because it's worse to have a bright flash in the dark than just
      // darkened screen in the light
      value: $nativeTheme.value === "unknown" ? "dark" : $nativeTheme.value,
    }),
  });

  const $preferredTheme = toRef($themeSettings.value, "preference");

  const $cosmetics = useCosmetics();

  const $activeTheme = toRef($themeSettings.value, "value");

  useHead({ htmlAttrs: { class: () => [`${$activeTheme.value}-mode`] } });

  function syncTheme() {
    const preferredTheme = $preferredTheme.value;

    if (preferredTheme === "system") {
      const nativeTheme = $nativeTheme.value;

      if (nativeTheme === "unknown" || nativeTheme === "dark") {
        let { preferredDarkTheme } = $cosmetics.value;
        if (preferredDarkTheme === "system") {
          // fail safe in case the user is messing with internals
          preferredDarkTheme = "dark";
        }

        $activeTheme.value = preferredDarkTheme;
      } else {
        $activeTheme.value = nativeTheme;
      }

      return;
    }

    $activeTheme.value = preferredTheme;
  }

  if (
    import.meta.server &&
    $preferredTheme.value === "system" &&
    $nativeTheme.value !== "unknown"
  ) {
    // take advantage of the client hint
    syncTheme();
  }

  if (import.meta.client) {
    const $clientReady = ref(false);

    nuxtApp.hook("app:suspense:resolve", () => {
      $clientReady.value = true;
    });

    watchEffect(() => $clientReady.value && syncTheme());
  }

  function cycle() {
    $preferredTheme.value = DarkThemes.includes($activeTheme.value)
      ? "light"
      : $cosmetics.value.preferredDarkTheme;
  }

  return {
    provide: {
      theme: reactive({
        preferred: $preferredTheme,
        active: $activeTheme,
        native: $nativeTheme,
        cycle,
      }),
    },
  };
});
