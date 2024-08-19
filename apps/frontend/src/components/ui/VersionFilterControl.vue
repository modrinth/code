<template>
  <div class="card flex-card">
    <span class="text-lg font-bold text-contrast">Filter</span>
    <div class="flex items-center gap-2">
      <div class="iconified-input w-full">
        <label class="hidden" for="search">Search</label>
        <SearchIcon aria-hidden="true" />
        <input
          id="search"
          v-model="queryFilter"
          name="search"
          type="search"
          placeholder="Search filters..."
          autocomplete="off"
        />
      </div>
      <button
        v-if="Object.keys(selectedFilters).length !== 0"
        class="btn icon-only"
        @click="clearFilters"
      >
        <FilterXIcon />
      </button>
    </div>
    <Accordion
      v-for="(value, key) in filters"
      :key="key"
      :open-by-default="true"
      type="transparent"
    >
      <template #title>
        <template v-if="key === 'gameVersion'"> Game versions </template>
        <template v-else>
          {{ $capitalizeString(key) }}
        </template>
      </template>

      <div>
        <ScrollablePanel :class="{ 'h-[18rem]': value.length > 4 }">
          <div class="mr-2 flex flex-col gap-1">
            <ButtonStyled
              v-for="filter in value"
              :key="filter"
              :type="selectedFilters[key]?.includes(filter) ? 'standard' : 'transparent'"
            >
              <button
                class="!mr-2 flex !w-full items-center !justify-normal !gap-2 !p-2 !py-1 !pl-8"
                @click="toggleFilter(key, filter)"
              >
                <span v-if="filter === 'release'" class="h-2 w-2 rounded-full bg-brand" />
                <span v-else-if="filter === 'beta'" class="h-2 w-2 rounded-full bg-orange" />
                <span v-else-if="filter === 'alpha'" class="h-2 w-2 rounded-full bg-red" />
                <span class="text-sm text-secondary">{{ $capitalizeString(filter) }}</span>
              </button>
            </ButtonStyled>
          </div>
        </ScrollablePanel>
        <Checkbox
          v-if="key === 'gameVersion'"
          v-model="showSnapshots"
          class="mx-1 ml-4"
          :label="`Show all versions`"
        />
      </div>
    </Accordion>
  </div>
</template>

<script setup>
import { FilterXIcon, SearchIcon } from "@modrinth/assets";
import { ButtonStyled, ScrollablePanel } from "@modrinth/ui";
import Checkbox from "~/components/ui/Checkbox.vue";
import Accordion from "~/components/ui/Accordion.vue";

const props = defineProps({
  versions: {
    type: Array,
    default() {
      return [];
    },
  },
});
const emit = defineEmits(["switch-page"]);

const route = useNativeRoute();
const router = useNativeRouter();

const tags = useTags();

const queryFilter = ref("");
const showSnapshots = ref(false);
const filters = computed(() => {
  const filters = {};

  const tempLoaders = new Set();
  const tempVersions = new Set();
  const tempReleaseChannels = new Set();

  for (const version of props.versions) {
    for (const loader of version.loaders) {
      tempLoaders.add(loader);
    }
    for (const gameVersion of version.game_versions) {
      tempVersions.add(gameVersion);
    }
    tempReleaseChannels.add(version.version_type);
  }

  if (tempReleaseChannels.size > 0) {
    filters.type = Array.from(tempReleaseChannels);
  }
  if (tempVersions.size > 0) {
    const gameVersions = tags.value.gameVersions.filter((x) => tempVersions.has(x.version));

    filters.gameVersion = gameVersions
      .filter((x) => (showSnapshots.value ? true : x.version_type === "release"))
      .map((x) => x.version);
  }
  if (tempLoaders.size > 0) {
    filters.platform = Array.from(tempLoaders);
  }

  const filteredObj = {};

  for (const [key, value] of Object.entries(filters)) {
    const filters = queryFilter.value
      ? value.filter((x) => x.toLowerCase().includes(queryFilter.value.toLowerCase()))
      : value;

    if (filters.length > 1) {
      filteredObj[key] = filters;
    }
  }

  return filteredObj;
});

const selectedFilters = ref({});

if (route.query.type) {
  selectedFilters.value.type = getArrayOrString(route.query.type);
}
if (route.query.gameVersion) {
  selectedFilters.value.gameVersion = getArrayOrString(route.query.gameVersion);
}
if (route.query.platform) {
  selectedFilters.value.platform = getArrayOrString(route.query.platform);
}

async function toggleFilters(type, filters) {
  for (const filter of filters) {
    await toggleFilter(type, filter);
  }

  await router.replace({
    query: {
      ...route.query,
      type: selectedFilters.value.type,
      gameVersion: selectedFilters.value.gameVersion,
      platform: selectedFilters.value.platform,
    },
  });

  emit("switch-page", 1);
}

async function toggleFilter(type, filter, skipRouter) {
  if (!selectedFilters.value[type]) {
    selectedFilters.value[type] = [];
  }

  const index = selectedFilters.value[type].indexOf(filter);
  if (index !== -1) {
    selectedFilters.value[type].splice(index, 1);
  } else {
    selectedFilters.value[type].push(filter);
  }

  if (selectedFilters.value[type].length === 0) {
    delete selectedFilters.value[type];
  }

  if (!skipRouter) {
    await router.replace({
      query: {
        ...route.query,
        type: selectedFilters.value.type,
        gameVersion: selectedFilters.value.gameVersion,
        platform: selectedFilters.value.platform,
      },
    });

    emit("switch-page", 1);
  }
}

async function clearFilters() {
  selectedFilters.value = {};

  await router.replace({
    query: {
      ...route.query,
      type: undefined,
      gameVersion: undefined,
      platform: undefined,
    },
  });

  emit("switch-page", 1);
}

defineExpose({
  toggleFilter,
  toggleFilters,
});
</script>
