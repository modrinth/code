import type { CookieOptions } from "#app";

export type ProjectDisplayMode = "list" | "grid" | "gallery";
export type DarkColorTheme = "dark" | "oled" | "retro";

export interface NumberFlag {
  min: number;
  max: number;
}

export type BooleanFlag = boolean;

export type RadioFlag = ProjectDisplayMode | DarkColorTheme;

export type FlagValue = BooleanFlag; /* | NumberFlag | RadioFlag */

const validateValues = <K extends PropertyKey>(flags: Record<K, FlagValue>) => flags;

export const DEFAULT_FEATURE_FLAGS = validateValues({
  // Developer flags
  developerMode: false,
  showVersionFilesInTable: false,
  showAdsWithPlus: false,
  alwaysShowChecklistAsPopup: true,

  // Feature toggles
  projectTypesPrimaryNav: false,
  hidePlusPromoInUserMenu: false,
  oldProjectCards: true,
  newProjectCards: false,
  projectBackground: false,
  searchBackground: false,
  advancedDebugInfo: false,
  showProjectPageDownloadModalServersPromo: false,
  showProjectPageCreateServersTooltip: true,
  showProjectPageQuickServerButton: false,
  // advancedRendering: true,
  // externalLinksNewTab: true,
  // notUsingBlockers: false,
  // hideModrinthAppPromos: false,
  // preferredDarkTheme: 'dark',
  // hideStagingBanner: false,

  // Project display modes
  // modSearchDisplayMode: 'list',
  // pluginSearchDisplayMode: 'list',
  // resourcePackSearchDisplayMode: 'gallery',
  // modpackSearchDisplayMode: 'list',
  // shaderSearchDisplayMode: 'gallery',
  // dataPackSearchDisplayMode: 'list',
  // userProjectDisplayMode: 'list',
  // collectionProjectDisplayMode: 'list',
} as const);

export type FeatureFlag = keyof typeof DEFAULT_FEATURE_FLAGS;

export type AllFeatureFlags = {
  [key in FeatureFlag]: (typeof DEFAULT_FEATURE_FLAGS)[key];
};

export type PartialFeatureFlags = Partial<AllFeatureFlags>;

const COOKIE_OPTIONS = {
  maxAge: 60 * 60 * 24 * 365 * 10,
  sameSite: "lax",
  secure: true,
  httpOnly: false,
  path: "/",
} satisfies CookieOptions<PartialFeatureFlags>;

export const useFeatureFlags = () =>
  useState<AllFeatureFlags>("featureFlags", () => {
    const config = useRuntimeConfig();

    const savedFlags = useCookie<PartialFeatureFlags>("featureFlags", COOKIE_OPTIONS);

    if (!savedFlags.value) {
      savedFlags.value = {};
    }

    const flags: AllFeatureFlags = JSON.parse(JSON.stringify(DEFAULT_FEATURE_FLAGS));

    const overrides = config.public.featureFlagOverrides as PartialFeatureFlags;
    for (const key in overrides) {
      if (key in flags) {
        const flag = key as FeatureFlag;
        const value = overrides[flag] as (typeof flags)[FeatureFlag];
        flags[flag] = value;
      }
    }

    for (const key in savedFlags.value) {
      if (key in flags) {
        const flag = key as FeatureFlag;
        const value = savedFlags.value[flag] as (typeof flags)[FeatureFlag];
        flags[flag] = value;
      }
    }

    return flags;
  });

export const saveFeatureFlags = () => {
  const flags = useFeatureFlags();
  const cookie = useCookie<PartialFeatureFlags>("featureFlags", COOKIE_OPTIONS);
  cookie.value = flags.value;
};
