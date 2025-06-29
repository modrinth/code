<template>
  <div>
    <MessageBanner v-if="flags.developerMode" message-type="warning" class="developer-message">
      <CodeIcon class="inline-flex" />
      <IntlFormatted :message-id="developerModeBanner.description">
        <template #strong="{ children }">
          <strong>
            <component :is="() => normalizeChildren(children)" />
          </strong>
        </template>
      </IntlFormatted>
      <Button :action="() => disableDeveloperMode()">
        {{ formatMessage(developerModeBanner.deactivate) }}
      </Button>
    </MessageBanner>
    <section class="universal-card">
      <h2 class="text-2xl">{{ formatMessage(colorTheme.title) }}</h2>
      <p>{{ formatMessage(colorTheme.description) }}</p>
      <ThemeSelector
        :update-color-theme="updateColorTheme"
        :current-theme="theme.preferred"
        :theme-options="themeOptions"
        :system-theme-color="systemTheme"
      />
    </section>
    <section class="universal-card">
      <h2 class="text-2xl">{{ formatMessage(projectListLayouts.title) }}</h2>
      <p class="mb-4">{{ formatMessage(projectListLayouts.description) }}</p>
      <div class="project-lists">
        <div v-for="projectType in listTypes" :key="projectType.id + '-project-list-layouts'">
          <div class="label">
            <div class="label__title">
              {{
                projectListLayouts[projectType.id]
                  ? formatMessage(projectListLayouts[projectType.id])
                  : projectType.id
              }}
            </div>
          </div>
          <div class="project-list-layouts">
            <button
              class="preview-radio button-base"
              :class="{ selected: cosmetics.searchDisplayMode[projectType.id] === 'list' }"
              @click="() => (cosmetics.searchDisplayMode[projectType.id] = 'list')"
            >
              <div class="preview">
                <div class="layout-list-mode">
                  <div class="example-card card"></div>
                  <div class="example-card card"></div>
                  <div class="example-card card"></div>
                  <div class="example-card card"></div>
                </div>
              </div>
              <div class="label">
                <RadioButtonCheckedIcon
                  v-if="cosmetics.searchDisplayMode[projectType.id] === 'list'"
                  class="radio"
                />
                <RadioButtonIcon v-else class="radio" />
                Rows
              </div>
            </button>
            <button
              class="preview-radio button-base"
              :class="{ selected: cosmetics.searchDisplayMode[projectType.id] === 'grid' }"
              @click="() => (cosmetics.searchDisplayMode[projectType.id] = 'grid')"
            >
              <div class="preview">
                <div class="layout-grid-mode">
                  <div class="example-card card"></div>
                  <div class="example-card card"></div>
                  <div class="example-card card"></div>
                  <div class="example-card card"></div>
                  <div class="example-card card"></div>
                  <div class="example-card card"></div>
                </div>
              </div>
              <div class="label">
                <RadioButtonCheckedIcon
                  v-if="cosmetics.searchDisplayMode[projectType.id] === 'grid'"
                  class="radio"
                />
                <RadioButtonIcon v-else class="radio" />
                Grid
              </div>
            </button>
            <button
              class="preview-radio button-base"
              :class="{ selected: cosmetics.searchDisplayMode[projectType.id] === 'gallery' }"
              @click="() => (cosmetics.searchDisplayMode[projectType.id] = 'gallery')"
            >
              <div class="preview">
                <div class="layout-gallery-mode">
                  <div class="example-card card"></div>
                  <div class="example-card card"></div>
                  <div class="example-card card"></div>
                  <div class="example-card card"></div>
                </div>
              </div>
              <div class="label">
                <RadioButtonCheckedIcon
                  v-if="cosmetics.searchDisplayMode[projectType.id] === 'gallery'"
                  class="radio"
                />
                <RadioButtonIcon v-else class="radio" />
                Gallery
              </div>
            </button>
          </div>
        </div>
      </div>
    </section>
    <section class="universal-card">
      <h2 class="text-2xl">{{ formatMessage(toggleFeatures.title) }}</h2>
      <p class="mb-4">{{ formatMessage(toggleFeatures.description) }}</p>
      <div class="adjacent-input small">
        <label for="advanced-rendering">
          <span class="label__title">
            {{ formatMessage(toggleFeatures.advancedRenderingTitle) }}
          </span>
          <span class="label__description">
            {{ formatMessage(toggleFeatures.advancedRenderingDescription) }}
          </span>
        </label>
        <input
          id="advanced-rendering"
          v-model="cosmetics.advancedRendering"
          class="switch stylized-toggle"
          type="checkbox"
        />
      </div>
      <div class="adjacent-input small">
        <label for="external-links-new-tab">
          <span class="label__title">
            {{ formatMessage(toggleFeatures.externalLinksNewTabTitle) }}
          </span>
          <span class="label__description">
            {{ formatMessage(toggleFeatures.externalLinksNewTabDescription) }}
          </span>
        </label>
        <input
          id="external-links-new-tab"
          v-model="cosmetics.externalLinksNewTab"
          class="switch stylized-toggle"
          type="checkbox"
        />
      </div>
      <div v-if="false" class="adjacent-input small">
        <label for="modrinth-app-promos">
          <span class="label__title">
            {{ formatMessage(toggleFeatures.hideModrinthAppPromosTitle) }}
          </span>
          <span class="label__description">
            {{ formatMessage(toggleFeatures.hideModrinthAppPromosDescription) }}
          </span>
        </label>
        <input
          id="modrinth-app-promos"
          v-model="cosmetics.hideModrinthAppPromos"
          class="switch stylized-toggle"
          type="checkbox"
        />
      </div>
      <div class="adjacent-input small">
        <label for="search-layout-toggle">
          <span class="label__title">
            {{ formatMessage(toggleFeatures.rightAlignedFiltersSidebarTitle) }}
          </span>
          <span class="label__description">
            {{ formatMessage(toggleFeatures.rightAlignedFiltersSidebarDescription) }}
          </span>
        </label>
        <input
          id="search-layout-toggle"
          v-model="cosmetics.rightSearchLayout"
          class="switch stylized-toggle"
          type="checkbox"
        />
      </div>
      <div class="adjacent-input small">
        <label for="project-layout-toggle">
          <span class="label__title">
            {{ formatMessage(toggleFeatures.leftAlignedContentSidebarTitle) }}
          </span>
          <span class="label__description">
            {{ formatMessage(toggleFeatures.leftAlignedContentSidebarDescription) }}
          </span>
        </label>
        <input
          id="project-layout-toggle"
          v-model="cosmetics.leftContentLayout"
          class="switch stylized-toggle"
          type="checkbox"
        />
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { CodeIcon, RadioButtonCheckedIcon, RadioButtonIcon } from "@modrinth/assets";
import { Button, ThemeSelector } from "@modrinth/ui";
import MessageBanner from "~/components/ui/MessageBanner.vue";
import type { DisplayLocation } from "~/plugins/cosmetics";
import { formatProjectType } from "~/plugins/shorthands.js";
import { isDarkTheme, type Theme } from "~/plugins/theme/index.ts";

useHead({
  title: "Display settings - Modrinth",
});

const { formatMessage } = useVIntl();

const developerModeBanner = defineMessages({
  description: {
    id: "settings.display.banner.developer-mode.description",
    defaultMessage:
      "<strong>Developer mode</strong> is active. This will allow you to view the internal IDs of various things throughout Modrinth that may be helpful if you're a developer using the Modrinth API. Click on the Modrinth logo at the bottom of the page 5 times to toggle developer mode.",
  },
  deactivate: {
    id: "settings.display.banner.developer-mode.button",
    defaultMessage: "Deactivate developer mode",
  },
});

const colorTheme = defineMessages({
  title: {
    id: "settings.display.theme.title",
    defaultMessage: "Color theme",
  },
  description: {
    id: "settings.display.theme.description",
    defaultMessage: "Select your preferred color theme for Modrinth on this device.",
  },
});

const projectListLayouts = defineMessages({
  title: {
    id: "settings.display.project-list-layouts.title",
    defaultMessage: "Project list layouts",
  },
  description: {
    id: "settings.display.project-list-layouts.description",
    defaultMessage:
      "Select your preferred layout for each page that displays project lists on this device.",
  },
  mod: {
    id: "settings.display.project-list-layouts.mod",
    defaultMessage: "Mods page",
  },
  plugin: {
    id: "settings.display.project-list-layouts.plugin",
    defaultMessage: "Plugins page",
  },
  datapack: {
    id: "settings.display.project-list-layouts.datapack",
    defaultMessage: "Data Packs page",
  },
  shader: {
    id: "settings.display.project-list-layouts.shader",
    defaultMessage: "Shaders page",
  },
  resourcepack: {
    id: "settings.display.project-list-layouts.resourcepack",
    defaultMessage: "Resource Packs page",
  },
  modpack: {
    id: "settings.display.project-list-layouts.modpack",
    defaultMessage: "Modpacks page",
  },
  user: {
    id: "settings.display.project-list-layouts.user",
    defaultMessage: "User profile pages",
  },
  collection: {
    id: "settings.display.project-list.layouts.collection",
    defaultMessage: "Collection",
  },
});

const toggleFeatures = defineMessages({
  title: {
    id: "settings.display.flags.title",
    defaultMessage: "Toggle features",
  },
  description: {
    id: "settings.display.flags.description",
    defaultMessage: "Enable or disable certain features on this device.",
  },
  advancedRenderingTitle: {
    id: "settings.display.sidebar.advanced-rendering.title",
    defaultMessage: "Advanced rendering",
  },
  advancedRenderingDescription: {
    id: "settings.display.sidebar.advanced-rendering.description",
    defaultMessage:
      "Enables advanced rendering such as blur effects that may cause performance issues without hardware-accelerated rendering.",
  },
  externalLinksNewTabTitle: {
    id: "settings.display.sidebar.external-links-new-tab.title",
    defaultMessage: "Open external links in new tab",
  },
  externalLinksNewTabDescription: {
    id: "settings.display.sidebar.external-links-new-tab.description",
    defaultMessage:
      "Make links which go outside of Modrinth open in a new tab. No matter this setting, links on the same domain and in Markdown descriptions will open in the same tab, and links on ads and edit pages will open in a new tab.",
  },
  hideModrinthAppPromosTitle: {
    id: "settings.display.sidebar.hide-app-promos.title",
    defaultMessage: "Hide Modrinth App promotions",
  },
  hideModrinthAppPromosDescription: {
    id: "settings.display.sidebar.hide-app-promos.description",
    defaultMessage:
      'Hides the "Get Modrinth App" buttons from primary navigation. The Modrinth App page can still be found on the landing page or in the footer.',
  },
  rightAlignedFiltersSidebarTitle: {
    id: "settings.display.sidebar.right-aligned-filters-sidebar.title",
    defaultMessage: "Right-aligned filters sidebar on search pages",
  },
  rightAlignedFiltersSidebarDescription: {
    id: "settings.display.sidebar.right-aligned-filters-sidebar.description",
    defaultMessage: "Aligns the filters sidebar to the right of the search results.",
  },
  leftAlignedContentSidebarTitle: {
    id: "settings.display.sidebar.left-aligned-content-sidebar.title",
    defaultMessage: "Left-aligned sidebar on content pages",
  },
  leftAlignedContentSidebarDescription: {
    id: "settings.display.sidebar.right-aligned-content-sidebar.description",
    defaultMessage: "Aligns the sidebar to the left of the page's content.",
  },
});

const cosmetics = useCosmetics();
const flags = useFeatureFlags();
const tags = useTags();

const theme = useTheme();

// On the server the value of native theme can be 'unknown'. To hydrate
// correctly, we need to make sure we aren't using 'unknown' and values between
// server and client renders are in sync.

const serverSystemTheme = useState(() => {
  const theme_ = theme.native;
  if (theme_ === "unknown") return "light";
  return theme_;
});

const systemTheme = useMountedValue((mounted): Theme => {
  const systemTheme_ = mounted ? theme.native : serverSystemTheme.value;
  return systemTheme_ === "light" ? theme.preferences.light : theme.preferences.dark;
});

const themeOptions = computed(() => {
  const options: ("system" | Theme)[] = ["system", "light", "dark", "oled"];
  if (flags.value.developerMode || theme.preferred === "retro") {
    options.push("retro");
  }
  return options;
});

function updateColorTheme(value: Theme | "system") {
  if (value !== "system") {
    if (isDarkTheme(value)) {
      theme.preferences.dark = value;
    } else {
      theme.preferences.light = value;
    }
  }

  theme.preferred = value;
}

function disableDeveloperMode() {
  flags.value.developerMode = !flags.value.developerMode;
  saveFeatureFlags();
  addNotification({
    group: "main",
    title: "Developer mode deactivated",
    text: "Developer mode has been disabled",
    type: "success",
  });
}

const listTypes = computed(() => {
  const types = tags.value.projectTypes.map((type) => {
    return {
      id: type.id as DisplayLocation,
      name: formatProjectType(type.id) + "s",
      display: "the " + formatProjectType(type.id).toLowerCase() + "s search page",
    };
  });

  types.push({
    id: "user" as DisplayLocation,
    name: "User profiles",
    display: "user pages",
  });

  return types;
});
</script>
<style scoped lang="scss">
.project-lists {
  display: flex;
  flex-direction: column;
  gap: var(--gap-md);

  > :first-child .label__title {
    margin-top: 0;
  }

  .preview {
    --_layout-width: 7rem;
    --_layout-height: 4.5rem;
    --_layout-gap: 0.25rem;

    .example-card {
      border-radius: 0.5rem;
      width: var(--_layout-width);
      height: calc((var(--_layout-height) - 3 * var(--_layout-gap)) / 4);
      padding: 0;
    }

    .layout-list-mode {
      display: grid;
      grid-template-columns: 1fr;
      gap: var(--_layout-gap);
    }

    .layout-grid-mode {
      display: grid;
      grid-template-columns: 1fr 1fr 1fr;
      gap: var(--_layout-gap);

      .example-card {
        width: calc((var(--_layout-width) - 2 * var(--_layout-gap)) / 3);
        height: calc((var(--_layout-height) - var(--_layout-gap)) / 2);
      }
    }

    .layout-gallery-mode {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: var(--_layout-gap);

      .example-card {
        width: calc((var(--_layout-width) - var(--_layout-gap)) / 2);
        height: calc((var(--_layout-height) - var(--_layout-gap)) / 2);
      }
    }
  }
}

.project-list-layouts {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(9.5rem, 1fr));
  gap: var(--gap-lg);

  .preview-radio .example-card {
    border: 2px solid transparent;
  }

  .preview-radio.selected .example-card {
    border-color: var(--color-brand);
    background-color: var(--color-brand-highlight);
  }

  .preview {
    display: flex;
    align-items: center;
    justify-content: center;
  }
}

.developer-message {
  svg {
    vertical-align: middle;
    margin-bottom: 2px;
    margin-right: 0.5rem;
  }

  .btn {
    margin-top: var(--gap-sm);
  }
}
</style>
