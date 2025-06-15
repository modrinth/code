import type { DarkTheme } from "./theme/index.ts";

export type DisplayMode = "list" | "gallery" | "grid";

export type DisplayLocation =
  | "mod"
  | "plugin"
  | "resourcepack"
  | "modpack"
  | "shader"
  | "datapack"
  | "user"
  | "collection";

export type ChartType = "area" | "line";

export interface Cosmetics {
  rightSearchLayout: boolean;
  leftContentLayout: boolean;
  advancedRendering: boolean;
  externalLinksNewTab: boolean;
  notUsingBlockers: boolean;
  hideModrinthAppPromos: boolean;
  preferredDarkTheme: DarkTheme;
  searchDisplayMode: Record<DisplayLocation, DisplayMode>;
  hideStagingBanner: boolean;
  analyticsChartType: ChartType;
  analyticsProjectColors: boolean;
}

export default defineNuxtPlugin({
  name: "cosmetics",
  setup() {
    const cosmetics = useCookie<Cosmetics>("cosmetics", {
      maxAge: 60 * 60 * 24 * 365 * 10,
      sameSite: "lax",
      secure: true,
      httpOnly: false,
      path: "/",
      default: () => ({
        rightSearchLayout: false,
        leftContentLayout: false,
        advancedRendering: true,
        externalLinksNewTab: true,
        notUsingBlockers: false,
        hideModrinthAppPromos: false,
        preferredDarkTheme: "dark",
        searchDisplayMode: {
          mod: "list",
          plugin: "list",
          resourcepack: "gallery",
          modpack: "list",
          shader: "gallery",
          datapack: "list",
          user: "list",
          collection: "list",
        },
        analyticsChartType: "line",
        analyticsProjectColors: true,
        hideStagingBanner: false,
      }),
    });

    return { provide: { cosmetics } };
  },
});
