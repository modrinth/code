<template>
  <div class="flex h-full w-full flex-col overflow-clip p-2 pb-4">
    <div class="iconified-input w-full">
      <label class="hidden" for="search">Search</label>
      <SearchIcon aria-hidden="true" />
      <input
        id="search"
        v-model="queryFilter"
        name="search"
        type="search"
        :placeholder="`Search ${props.type}s...`"
        autocomplete="off"
        @keyup.enter="resetList"
      />
    </div>
    <div class="flex h-[93%] w-full flex-col">
      <div
        class="flex h-full w-full flex-col gap-2 overflow-y-scroll pt-2"
        v-if="mods && mods.hits.length > 0"
        ref="scrollContainer"
      >
        <div v-for="mod in mods.hits" :key="mod.title" class="rounded-lg p-2 hover:bg-divider-dark">
          <div class="flex cursor-pointer gap-2" @click="toggleMod(mod.project_id)">
            <UiAvatar :src="mod.icon_url" size="sm" />
            <div class="flex flex-col gap-1">
              <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">
                {{ mod.title }}
              </h1>
              <span class="text-sm font-semibold text-secondary">
                {{ mod.description.substring(0, 100) }}
                {{ mod.description.length > 100 ? "..." : "" }}
              </span>
            </div>
          </div>
          <div v-if="expandedMods[mod.project_id]" class="mt-2 flex items-center gap-2">
            <DropdownSelect
              id="version-select"
              v-model="selectedVersion"
              name="version-select"
              :options="expandedMods[mod.project_id].versions"
              placeholder="Select version..."
            />
            <Button icon-only @click="emits('select', mod, selectedVersion)">
              <ChevronRightIcon />
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from "vue";
import { ChevronRightIcon, SearchIcon } from "@modrinth/assets";
import { Button, DropdownSelect } from "@modrinth/ui";
import { useInfiniteScroll } from "@vueuse/core";

const emits = defineEmits(["select"]);

const props = defineProps<{
  type: "mod" | "modpack" | "plugin" | "datapack";
  server?: boolean;
}>();

const serverStore = useServerStore();
const route = useNativeRoute();
const config = useRuntimeConfig();
const prodOverride = await PyroAuthOverride();
const serverId = route.params.id as string;

const data = computed(() => serverStore.serverData[serverId]);

const scrollContainer = ref<HTMLElement | null>(null);
const pages = ref(1);
const page = ref(0);

const queryFilter = ref("");
const facets = ref<any>([]);

if (props.server === false && props.type !== "modpack") {
  facets.value.push(`["categories:${data.value?.loader?.toLocaleLowerCase()}"]`);
  facets.value.push(`["versions:${data.value?.mc_version}"]`);
}

facets.value.push(`["project_type:${props.type}"]`);

const buildFacetString = (facets: string[]) => {
  return "[" + facets.map((facet) => `${facet}`).join(",") + "]";
};

const mods = ref<any>({ hits: [] });
const modsStatus = ref("idle");

const loadMods = async () => {
  modsStatus.value = "loading";

  const newMods = (await useBaseFetch(
    `search?query=${queryFilter.value}&facets=${buildFacetString(facets.value)}&index=relevance&limit=100&offset=${page.value * 100}`,
    {},
    false,
    prodOverride,
  )) as any;
  pages.value = newMods.total_hits;
  mods.value.hits.push(...newMods.hits);
  modsStatus.value = "success";
};

const versions = reactive<{ [key: string]: any[] }>({});

const getVersions = async (project_id: string) => {
  if (!versions[project_id]) {
    const allVersions = (await useBaseFetch(
      `project/${project_id}/version`,
      {},
      false,
      prodOverride,
    )) as any;

    if (props.server === false && props.type !== "modpack") {
      versions[project_id] = allVersions
        .filter((x: any) => x.loaders.includes(data.value?.loader?.toLocaleLowerCase()))
        .filter((x: any) => x.game_versions.includes(data.value?.mc_version))
        .map((x: any) => x.version_number);
    } else {
      versions[project_id] = allVersions.map((x: any) => x.version_number);
    }
  }
  return versions[project_id];
};

const selectedVersion = ref("");

const expandedMods = reactive<{ [key: string]: { expanded: boolean; versions: any[] } }>({});

const toggleMod = async (modId: string) => {
  if (!expandedMods[modId]) {
    expandedMods[modId] = { expanded: false, versions: [] };
  }
  expandedMods[modId].expanded = !expandedMods[modId].expanded;
  if (expandedMods[modId].expanded && expandedMods[modId].versions.length === 0) {
    expandedMods[modId].versions = await getVersions(modId);
  }
  selectedVersion.value = "";
};

const loadMore = async () => {
  page.value++;
  await loadMods();
};

const { reset } = useInfiniteScroll(scrollContainer, async () => {
  if (page.value <= pages.value) {
    await loadMore();
  }
});

const resetList = () => {
  mods.value.hits = [];
  Object.keys(expandedMods).forEach((key) => delete expandedMods[key]);
  page.value = 0;
  selectedVersion.value = "";
  loadMods();
  reset();
};

onMounted(async () => {
  await loadMods();
});
</script>
