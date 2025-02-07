<template>
  <div class="experimental-styles-within flex w-full flex-col items-center gap-2">
    <ManySelect
      v-model="selectedPlatforms"
      :tooltip="
        filterOptions.platform.length < 2 && !disabled ? 'No other platforms available' : undefined
      "
      :options="filterOptions.platform"
      :dropdown-id="`${baseId}-platform`"
      search
      show-always
      class="w-full"
      :disabled="disabled || filterOptions.platform.length < 2"
      :dropdown-class="'w-full'"
      @change="updateFilters"
    >
      <slot name="platform">
        <FilterIcon class="h-5 w-5 text-secondary" />
        Platform
      </slot>
      <template #option="{ option }">
        {{ formatCategory(option) }}
      </template>
      <template v-if="hasAnyUnsupportedPlatforms" #footer>
        <Checkbox
          v-model="showSupportedPlatformsOnly"
          class="mx-1"
          :label="`Show ${type?.toLowerCase()} platforms only`"
        />
      </template>
    </ManySelect>
    <ManySelect
      v-model="selectedGameVersions"
      :tooltip="
        filterOptions.gameVersion.length < 2 && !disabled
          ? 'No other game versions available'
          : undefined
      "
      :options="filterOptions.gameVersion"
      :dropdown-id="`${baseId}-game-version`"
      search
      show-always
      class="w-full"
      :disabled="disabled || filterOptions.gameVersion.length < 2"
      :dropdown-class="'w-full'"
      @change="updateFilters"
    >
      <slot name="game-versions">
        <FilterIcon class="h-5 w-5 text-secondary" />
        Game versions
      </slot>
      <template v-if="hasAnySnapshots" #footer>
        <Checkbox v-model="showSnapshots" class="mx-1" :label="`Show all versions`" />
      </template>
    </ManySelect>
  </div>
</template>

<script setup lang="ts">
import { FilterIcon } from "@modrinth/assets";
import { type Version, formatCategory, type GameVersionTag } from "@modrinth/utils";
import { ref, computed } from "vue";
import { useRoute } from "vue-router";
import ManySelect from "@modrinth/ui/src/components/base/ManySelect.vue";
import Checkbox from "@modrinth/ui/src/components/base/Checkbox.vue";

export type ListedGameVersion = {
  name: string;
  release: boolean;
};

export type ListedPlatform = {
  name: string;
  isType: boolean;
};

const props = defineProps<{
  versions: Version[];
  gameVersions: GameVersionTag[];
  listedGameVersions: ListedGameVersion[];
  listedPlatforms: ListedPlatform[];
  baseId?: string;
  type: "Mod" | "Plugin";
  platformTags: {
    name: string;
    supported_project_types: string[];
  }[];
  disabled?: boolean;
}>();

const emit = defineEmits(["update:query"]);
const route = useRoute();

const showSnapshots = ref(false);
const hasAnySnapshots = computed(() => {
  return props.versions.some((x) =>
    props.gameVersions.some(
      (y) => y.version_type !== "release" && x.game_versions.includes(y.version),
    ),
  );
});

const hasOnlySnapshots = computed(() => {
  return props.versions.every((version) => {
    return version.game_versions.every((gv) => {
      const matched = props.gameVersions.find((tag) => tag.version === gv);
      return matched && matched.version_type !== "release";
    });
  });
});

const hasAnyUnsupportedPlatforms = computed(() => {
  return props.listedPlatforms.some((x) => !x.isType);
});

const hasOnlyUnsupportedPlatforms = computed(() => {
  return props.listedPlatforms.every((x) => !x.isType);
});

const showSupportedPlatformsOnly = ref(true);

const filterOptions = computed(() => {
  const filters: Record<"gameVersion" | "platform", string[]> = {
    gameVersion: [],
    platform: [],
  };

  filters.gameVersion = props.listedGameVersions
    .filter((x) => {
      return showSnapshots.value || hasOnlySnapshots.value ? true : x.release;
    })
    .map((x) => x.name);

  filters.platform = props.listedPlatforms
    .filter((x) => {
      return !showSupportedPlatformsOnly.value || hasOnlyUnsupportedPlatforms.value
        ? true
        : x.isType;
    })
    .map((x) => x.name);

  return filters;
});

const selectedGameVersions = ref<string[]>([]);
const selectedPlatforms = ref<string[]>([]);

selectedGameVersions.value = route.query.g ? getArrayOrString(route.query.g) : [];
selectedPlatforms.value = route.query.l ? getArrayOrString(route.query.l) : [];

function updateFilters() {
  emit("update:query", {
    g: selectedGameVersions.value,
    l: selectedPlatforms.value,
  });
}

defineExpose({
  selectedGameVersions,
  selectedPlatforms,
});

function getArrayOrString(x: string | (string | null)[]): string[] {
  if (typeof x === "string") {
    return [x];
  } else {
    return x.filter((item): item is string => item !== null);
  }
}
</script>

<style></style>
