import { ref } from "vue";
import { isDarkTheme } from "./themes.ts";
import { useNativeTheme } from "./native-theme.ts";
import { usePreferredThemes } from "./preferred-theme.ts";
import { useThemeSettings } from "./theme-settings.ts";

export * from "./themes.ts";

export default defineNuxtPlugin({
  name: "theme",
  dependsOn: ["cosmetics"],
  setup(nuxtApp) {
    const $nativeTheme = useNativeTheme();

    const $preferredThemes = usePreferredThemes();

    function getPreferredNativeTheme() {
      const nativeTheme = $nativeTheme.value;
      switch (nativeTheme) {
        case "light":
          return $preferredThemes.light;
        case "dark":
        case "unknown":
          if (import.meta.dev && import.meta.server && nativeTheme === "unknown") {
            console.warn(
              "[theme] no client hint is available for request, using dark theme as default",
            );
          }

          return $preferredThemes.dark;
      }
    }

    const $settings = useThemeSettings(() => getPreferredNativeTheme());

    useHead({ htmlAttrs: { class: () => [`${$settings.active}-mode`] } });

    function syncTheme() {
      $settings.active =
        $settings.preferred === "system" ? getPreferredNativeTheme() : $settings.preferred;
    }

    if (
      import.meta.server &&
      $settings.preferred === "system" &&
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
      const nextTheme = isDarkTheme($settings.active)
        ? $preferredThemes.light
        : $preferredThemes.dark;

      $settings.preferred = nextTheme;

      return nextTheme;
    }

    return {
      provide: {
        theme: reactive({
          ...toRefs($settings),
          /**
           * Preferred themes for each mode.
           */
          preferences: $preferredThemes,
          /**
           * Current native (system) theme provided through client hint header or
           * `prefers-color-scheme` media query.
           */
          native: $nativeTheme,
          cycle,
        }),
      },
    };
  },
});
