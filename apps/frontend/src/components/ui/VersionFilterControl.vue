<template>
  <div class="card flex-card experimental-styles-within">
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
    <div
      v-for="(value, key, index) in filters"
      :key="key"
      :class="`border-0 border-b border-solid border-button-bg py-2 last:border-b-0`"
    >
      <button
        class="flex !w-full bg-transparent px-0 py-2 font-extrabold text-contrast transition-all active:scale-[0.98]"
        @click="
          () => {
            filterAccordions[index].isOpen
              ? filterAccordions[index].close()
              : filterAccordions[index].open();
          }
        "
      >
        <template v-if="key === 'gameVersion'"> Game versions </template>
        <template v-else>
          {{ $capitalizeString(key) }}
        </template>
        <DropdownIcon
          class="ml-auto h-5 w-5 transition-transform"
          :class="{ 'rotate-180': filterAccordions[index]?.isOpen }"
        />
      </button>
      <Accordion ref="filterAccordions" :open-by-default="true">
        <ScrollablePanel
          :class="{ 'h-[18rem]': value.length >= 8 && key === 'gameVersion' }"
          :no-max-height="key !== 'gameVersion'"
        >
          <div class="mr-1 flex flex-col gap-1">
            <div v-for="filter in value" :key="filter" class="group flex gap-1">
              <button
                :class="`flex !w-full items-center gap-2 truncate rounded-xl px-2 py-1 text-sm font-semibold transition-all active:scale-[0.98] ${selectedFilters[key]?.includes(filter) ? 'bg-brand-highlight text-contrast hover:brightness-125' : 'bg-transparent text-secondary hover:bg-button-bg'}`"
                @click="toggleFilter(key, filter)"
              >
                <span v-if="filter === 'release'" class="h-2 w-2 rounded-full bg-brand" />
                <span v-else-if="filter === 'beta'" class="h-2 w-2 rounded-full bg-orange" />
                <span v-else-if="filter === 'alpha'" class="h-2 w-2 rounded-full bg-red" />
                <span class="truncate text-sm">{{ $formatCategory(filter) }}</span>
              </button>
            </div>
          </div>
        </ScrollablePanel>
        <Checkbox
          v-if="key === 'gameVersion'"
          v-model="showSnapshots"
          class="mx-2"
          :label="`Show all versions`"
        />
      </Accordion>
    </div>
  </div>
</template>

<script setup>
import { DropdownIcon, FilterXIcon, SearchIcon } from "@modrinth/assets";
import { ScrollablePanel, Checkbox } from "@modrinth/ui";
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

const filterAccordions = ref([]);

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

    if (filters.length > 0) {
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
  filters,
  selectedFilters,
  clearFilters,
});
</script>
