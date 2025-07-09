<template>
  <NewModal ref="modModal" :header="`Editing ${type.toLocaleLowerCase()} version`">
    <template #title>
      <div class="flex min-w-full items-center gap-2 md:w-[calc(420px-5.5rem)]">
        <Avatar :src="modDetails?.icon_url" size="48px" :alt="`${modDetails?.name} Icon`" />
        <span class="truncate text-xl font-extrabold text-contrast">{{ modDetails?.name }}</span>
      </div>
    </template>
    <div class="flex flex-col gap-2 md:w-[420px]">
      <div class="flex flex-col gap-2">
        <template v-if="versionsLoading">
          <div class="flex items-center gap-2">
            <div class="w-fit animate-pulse select-none rounded-md bg-button-bg font-semibold">
              <span class="opacity-0" aria-hidden="true">{{ type }} version</span>
            </div>
            <div class="min-h-[22px] min-w-[140px] animate-pulse rounded-full bg-button-bg" />
          </div>
          <div class="min-h-9 w-full animate-pulse rounded-xl bg-button-bg" />
          <div class="w-fit animate-pulse select-none rounded-md bg-button-bg">
            <span class="ml-6 opacity-0" aria-hidden="true">
              Show any beta and alpha releases
            </span>
          </div>
        </template>

        <template v-else>
          <div class="flex justify-between">
            <div class="flex items-center gap-2">
              <div class="font-semibold text-contrast">{{ type }} version</div>
              <NuxtLink
                class="flex cursor-pointer items-center gap-1 bg-transparent p-0"
                @click="
                  versionFilter &&
                    (unlockFilterAccordion.isOpen
                      ? unlockFilterAccordion.close()
                      : unlockFilterAccordion.open())
                "
              >
                <TagItem
                  v-if="formattedVersions.game_versions.length > 0"
                  v-tooltip="formattedVersions.game_versions.join(', ')"
                  :style="`--_color: var(--color-green)`"
                >
                  {{ formattedVersions.game_versions[0] }}
                </TagItem>
                <TagItem
                  v-if="formattedVersions.loaders.length > 0"
                  v-tooltip="formattedVersions.loaders.join(', ')"
                  :style="`--_color: var(--color-platform-${formattedVersions.loaders[0].toLowerCase()})`"
                >
                  {{ formattedVersions.loaders[0] }}
                </TagItem>
                <DropdownIcon
                  :class="[
                    'transition-all duration-200 ease-in-out',
                    { 'rotate-180': unlockFilterAccordion.isOpen },
                    { 'opacity-0': !versionFilter },
                  ]"
                />
              </NuxtLink>
            </div>
          </div>
          <UiServersTeleportDropdownMenu
            v-model="selectedVersion"
            name="Project"
            :options="filteredVersions"
            placeholder="No valid versions found"
            class="!min-w-full"
            :disabled="filteredVersions.length === 0"
            :display-name="
              (version) => (typeof version === 'object' ? version?.version_number : version)
            "
          />
          <Checkbox v-model="showBetaAlphaReleases"> Show any beta and alpha releases </Checkbox>
        </template>
      </div>

      <Accordion
        ref="unlockFilterAccordion"
        :open-by-default="!versionFilter"
        :class="[
          versionFilter ? '' : '!border-solid border-orange bg-bg-orange !text-contrast',
          'flex flex-col gap-2 rounded-2xl border-2 border-dashed border-divider p-3 transition-all',
        ]"
      >
        <p class="m-0 items-center font-bold">
          <span>
            {{
              noCompatibleVersions
                ? `No compatible versions of this ${type.toLowerCase()} were found`
                : versionFilter
                  ? "Game version and platform is provided by the server"
                  : "Incompatible game version and platform versions are unlocked"
            }}
          </span>
        </p>
        <p class="m-0 text-sm">
          {{
            noCompatibleVersions
              ? `No versions compatible with your server were found. You can still select any available version.`
              : versionFilter
                ? `Unlocking this filter may allow you to change this ${type.toLowerCase()}
            to an incompatible version.`
                : "You might see versions listed that aren't compatible with your server configuration."
          }}
        </p>
        <ContentVersionFilter
          v-if="currentVersions"
          ref="filtersRef"
          :versions="currentVersions"
          :game-versions="tags.gameVersions"
          :select-classes="'w-full'"
          :type="type"
          :disabled="versionFilter"
          :platform-tags="tags.loaders"
          :listed-game-versions="gameVersions"
          :listed-platforms="platforms"
          @update:query="updateFiltersFromUi($event)"
          @vue:mounted="updateFiltersToUi"
        >
          <template #platform>
            <LoaderIcon
              v-if="filtersRef?.selectedPlatforms.length === 0"
              :loader="'Vanilla'"
              class="size-5 flex-none"
            />
            <svg
              v-else
              class="size-5 flex-none"
              v-html="tags.loaders.find((x) => x.name === filtersRef?.selectedPlatforms[0])?.icon"
            ></svg>

            <div class="w-full truncate text-left">
              {{
                filtersRef?.selectedPlatforms.length === 0
                  ? "All platforms"
                  : filtersRef?.selectedPlatforms.map((x) => formatCategory(x)).join(", ")
              }}
            </div>
          </template>
          <template #game-versions>
            <GameIcon class="size-5 flex-none" />
            <div class="w-full truncate text-left">
              {{
                filtersRef?.selectedGameVersions.length === 0
                  ? "All game versions"
                  : filtersRef?.selectedGameVersions.join(", ")
              }}
            </div>
          </template>
        </ContentVersionFilter>

        <ButtonStyled v-if="!noCompatibleVersions" color-fill="text">
          <button
            class="w-full"
            :disabled="gameVersions.length < 2 && platforms.length < 2"
            @click="
              () => {
                versionFilter = !versionFilter;
                setInitialFilters();
                updateFiltersToUi();
              }
            "
          >
            <LockOpenIcon />
            {{
              gameVersions.length < 2 && platforms.length < 2
                ? "No other platforms or versions available"
                : versionFilter
                  ? "Unlock"
                  : "Return to compatibility"
            }}
          </button>
        </ButtonStyled>
      </Accordion>

      <Admonition
        v-if="versionsError"
        type="critical"
        header="Failed to load versions"
        class="mb-2"
      >
        <div>
          <span>
            Something went wrong trying to load versions for this {{ type.toLocaleLowerCase() }}.
            Please try again later or contact support if the issue persists.
          </span>
          <CopyCode class="!mt-2 !break-all" :text="versionsError" />
        </div>
      </Admonition>

      <Admonition
        v-else-if="props.modPack"
        type="warning"
        header="Changing version may cause issues"
        class="mb-2"
      >
        Your server was created using a modpack. It's recommended to use the modpack's version of
        the mod.
        <NuxtLink
          class="mt-2 flex items-center gap-1"
          :to="`/servers/manage/${props.serverId}/options/loader`"
          target="_blank"
        >
          <ExternalIcon class="size-5 flex-none"></ExternalIcon> Modify modpack version
        </NuxtLink>
      </Admonition>

      <div class="flex flex-row items-center gap-4">
        <ButtonStyled color="brand">
          <button
            :disabled="versionsLoading || selectedVersion.id === modDetails?.version_id"
            @click="emitChangeModVersion"
          >
            <CheckIcon />
            Install
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button @click="modModal.hide()">
            <XIcon />
            Cancel
          </button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>
</template>

<script setup lang="ts">
import {
  DropdownIcon,
  XIcon,
  CheckIcon,
  LockOpenIcon,
  GameIcon,
  ExternalIcon,
} from "@modrinth/assets";
import { Admonition, Avatar, ButtonStyled, CopyCode, NewModal } from "@modrinth/ui";
import TagItem from "@modrinth/ui/src/components/base/TagItem.vue";
import { ref, computed } from "vue";
import { formatCategory, formatVersionsForDisplay, type Mod, type Version } from "@modrinth/utils";
import Accordion from "~/components/ui/Accordion.vue";
import Checkbox from "~/components/ui/Checkbox.vue";
import ContentVersionFilter, {
  type ListedGameVersion,
  type ListedPlatform,
} from "~/components/ui/servers/ContentVersionFilter.vue";
import LoaderIcon from "~/components/ui/servers/icons/LoaderIcon.vue";

const props = defineProps<{
  type: "Mod" | "Plugin";
  loader: string;
  gameVersion: string;
  modPack: boolean;
  serverId: string;
}>();

interface ContentItem extends Mod {
  changing?: boolean;
}

interface EditVersion extends Version {
  installed: boolean;
  upgrade?: boolean;
}

const modModal = ref();
const modDetails = ref<ContentItem>();
const currentVersions = ref<EditVersion[] | null>(null);
const versionsLoading = ref(false);
const versionsError = ref("");
const showBetaAlphaReleases = ref(false);
const unlockFilterAccordion = ref();
const versionFilter = ref(true);
const tags = useTags();
const noCompatibleVersions = ref(false);

const { pluginLoaders, modLoaders } = tags.value.loaders.reduce(
  (acc, tag) => {
    if (tag.supported_project_types.includes("plugin")) {
      acc.pluginLoaders.push(tag.name);
    }
    if (tag.supported_project_types.includes("mod")) {
      acc.modLoaders.push(tag.name);
    }
    return acc;
  },
  { pluginLoaders: [] as string[], modLoaders: [] as string[] },
);

const selectedVersion = ref();
const filtersRef: Ref<InstanceType<typeof ContentVersionFilter> | null> = ref(null);
interface SelectedContentFilters {
  selectedGameVersions: string[];
  selectedPlatforms: string[];
}
const selectedFilters = ref<SelectedContentFilters>({
  selectedGameVersions: [],
  selectedPlatforms: [],
});

const backwardCompatPlatformMap = {
  purpur: ["purpur", "paper", "spigot", "bukkit"],
  paper: ["paper", "spigot", "bukkit"],
  spigot: ["spigot", "bukkit"],
};

const platforms = ref<ListedPlatform[]>([]);
const gameVersions = ref<ListedGameVersion[]>([]);
const initPlatform = ref<string>("");

const setInitialFilters = () => {
  selectedFilters.value = {
    selectedGameVersions: [
      gameVersions.value.find((version) => version.name === props.gameVersion)?.name ??
        gameVersions.value.find((version) => version.release)?.name ??
        gameVersions.value[0]?.name,
    ],
    selectedPlatforms: [initPlatform.value],
  };
};

const updateFiltersToUi = () => {
  if (!filtersRef.value) return;
  filtersRef.value.selectedGameVersions = selectedFilters.value.selectedGameVersions;
  filtersRef.value.selectedPlatforms = selectedFilters.value.selectedPlatforms;

  selectedVersion.value = filteredVersions.value[0];
};

const updateFiltersFromUi = (event: { g: string[]; l: string[] }) => {
  selectedFilters.value = {
    selectedGameVersions: event.g,
    selectedPlatforms: event.l,
  };
};

const filteredVersions = computed(() => {
  if (!currentVersions.value) return [];

  const versionsWithoutReleaseFilter = currentVersions.value.filter((version: EditVersion) => {
    if (version.installed) return true;
    return (
      filtersRef.value?.selectedPlatforms.every((platform) =>
        (
          backwardCompatPlatformMap[platform as keyof typeof backwardCompatPlatformMap] || [
            platform,
          ]
        ).some((loader) => version.loaders.includes(loader)),
      ) &&
      filtersRef.value?.selectedGameVersions.every((gameVersion) =>
        version.game_versions.includes(gameVersion),
      )
    );
  });

  const versionTypes = new Set(
    versionsWithoutReleaseFilter.map((v: EditVersion) => v.version_type),
  );
  const releaseVersions = versionTypes.has("release");
  const betaVersions = versionTypes.has("beta");
  const alphaVersions = versionTypes.has("alpha");

  const versions = versionsWithoutReleaseFilter.filter((version: EditVersion) => {
    if (showBetaAlphaReleases.value || version.installed) return true;
    return releaseVersions
      ? version.version_type === "release"
      : betaVersions
        ? version.version_type === "beta"
        : alphaVersions
          ? version.version_type === "alpha"
          : false;
  });

  return versions.map((version: EditVersion) => {
    let suffix = "";

    if (version.version_type === "alpha" && releaseVersions && betaVersions) {
      suffix += " (alpha)";
    } else if (version.version_type === "beta" && releaseVersions) {
      suffix += " (beta)";
    }

    return {
      ...version,
      version_number: version.version_number + suffix,
    };
  });
});

const formattedVersions = computed(() => {
  return {
    game_versions: formatVersionsForDisplay(
      selectedVersion.value?.game_versions || [],
      tags.value.gameVersions,
    ),
    loaders: (selectedVersion.value?.loaders || [])
      .sort((firstLoader: string, secondLoader: string) => {
        const loaderList = backwardCompatPlatformMap[
          props.loader as keyof typeof backwardCompatPlatformMap
        ] || [props.loader];

        const firstLoaderPosition = loaderList.indexOf(firstLoader.toLowerCase());
        const secondLoaderPosition = loaderList.indexOf(secondLoader.toLowerCase());

        if (firstLoaderPosition === -1 && secondLoaderPosition === -1) return 0;
        if (firstLoaderPosition === -1) return 1;
        if (secondLoaderPosition === -1) return -1;
        return firstLoaderPosition - secondLoaderPosition;
      })
      .map((loader: string) => formatCategory(loader)),
  };
});

async function show(mod: ContentItem) {
  versionFilter.value = true;
  modModal.value.show();
  versionsLoading.value = true;
  modDetails.value = mod;
  versionsError.value = "";
  currentVersions.value = null;

  try {
    const result = await useBaseFetch(`project/${mod.project_id}/version`, {}, false);
    if (
      Array.isArray(result) &&
      result.every(
        (item) =>
          "id" in item &&
          "version_number" in item &&
          "version_type" in item &&
          "loaders" in item &&
          "game_versions" in item,
      )
    ) {
      currentVersions.value = result as EditVersion[];
    } else {
      throw new Error("Invalid version data received.");
    }

    // find the installed version and move it to the top of the list
    const currentModIndex = currentVersions.value.findIndex(
      (item: { id: string }) => item.id === mod.version_id,
    );
    if (currentModIndex === -1) {
      currentVersions.value[currentModIndex] = {
        ...currentVersions.value[currentModIndex],
        installed: true,
        version_number: `${mod.version_number} (current) (external)`,
      };
    } else {
      currentVersions.value[currentModIndex].version_number = `${mod.version_number} (current)`;
      currentVersions.value[currentModIndex].installed = true;
    }

    // initially filter the platform and game versions for the server config
    const platformSet = new Set<string>();
    const gameVersionSet = new Set<string>();
    for (const version of currentVersions.value) {
      for (const loader of version.loaders) {
        platformSet.add(loader);
      }
      for (const gameVersion of version.game_versions) {
        gameVersionSet.add(gameVersion);
      }
    }
    if (gameVersionSet.size > 0) {
      const filteredGameVersions = tags.value.gameVersions.filter((x) =>
        gameVersionSet.has(x.version),
      );

      gameVersions.value = filteredGameVersions.map((x) => ({
        name: x.version,
        release: x.version_type === "release",
      }));
    }
    if (platformSet.size > 0) {
      const tempPlatforms = Array.from(platformSet).map((platform) => ({
        name: platform,
        isType:
          props.type === "Plugin"
            ? pluginLoaders.includes(platform)
            : props.type === "Mod"
              ? modLoaders.includes(platform)
              : false,
      }));
      platforms.value = tempPlatforms;
    }

    // set default platform
    const defaultPlatform = Array.from(platformSet)[0];
    initPlatform.value = platformSet.has(props.loader)
      ? props.loader
      : props.loader in backwardCompatPlatformMap
        ? backwardCompatPlatformMap[props.loader as keyof typeof backwardCompatPlatformMap].find(
            (p) => platformSet.has(p),
          ) || defaultPlatform
        : defaultPlatform;

    // check if there's nothing compatible with the server config
    noCompatibleVersions.value =
      !platforms.value.some((p) => p.isType) ||
      !gameVersions.value.some((v) => v.name === props.gameVersion);

    if (noCompatibleVersions.value) {
      unlockFilterAccordion.value.open();
      versionFilter.value = false;
    }

    setInitialFilters();
    versionsLoading.value = false;
  } catch (error) {
    console.error("Error loading versions:", error);
    versionsError.value = error instanceof Error ? error.message : "Unknown";
  }
}

const emit = defineEmits<{
  changeVersion: [string];
}>();

function emitChangeModVersion() {
  if (!selectedVersion.value) return;
  emit("changeVersion", selectedVersion.value.id.toString());
}

defineExpose({
  show,
  hide: () => modModal.value.hide(),
});
</script>
