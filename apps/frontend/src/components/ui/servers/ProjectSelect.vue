<template>
  <div class="flex h-[400px] w-full max-w-xl flex-col overflow-hidden">
    <div class="iconified-input mb-4 w-full">
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
    <div class="flex h-full w-full flex-col">
      <div
        v-if="mods && mods.hits.length > 0"
        ref="scrollContainer"
        class="flex h-full w-full flex-col gap-2 overflow-y-scroll"
      >
        <div v-for="mod in mods.hits" :key="mod.title" class="rounded-lg px-2 py-2 hover:bg-bg">
          <div class="flex cursor-pointer gap-2" @click="toggleMod(mod.project_id)">
            <UiAvatar :src="mod.icon_url" class="!h-12 !min-h-12 !w-12 !min-w-12" />
            <div class="flex flex-col gap-1">
              <h1 class="m-0 text-2xl font-bold leading-none text-contrast">
                {{ mod.title }}
              </h1>
              <span class="text-sm text-secondary">
                {{ mod.description.substring(0, 100) }}
                {{ mod.description.length > 100 ? "..." : "" }}
              </span>
            </div>
          </div>
          <div v-if="expandedMods[mod.project_id]" class="mt-2 flex items-center gap-2">
            <DropdownSelect
              id="version-select"
              v-model="selectedVersions[mod.project_id]"
              name="version-select"
              :options="expandedMods[mod.project_id].versions"
              placeholder="Select version..."
            />
            <Button icon-only @click="emits('select', mod, selectedVersions[mod.project_id])">
              <ChevronRightIcon />
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ChevronRightIcon, SearchIcon } from "@modrinth/assets";
import { Button, DropdownSelect } from "@modrinth/ui";
import { useInfiniteScroll } from "@vueuse/core";

const emits = defineEmits(["select"]);

const props = defineProps<{
  type: "mod" | "modpack" | "plugin" | "datapack";
  isserver?: boolean;
}>();

const route = useNativeRoute();
const serverId = route.params.id as string;
const server = serverId ? await usePyroServer(serverId, ["general"]) : null;

const data = computed(() => (serverId ? server?.general : null));

const scrollContainer = ref<HTMLElement | null>(null);
const pages = ref(1);
const page = ref(0);

const queryFilter = ref("");
const facets = ref<any>([]);

if (props.isserver === false && props.type !== "modpack") {
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
    `search?query=${queryFilter.value}&facets=${buildFacetString(facets.value)}&index=relevance&limit=25&offset=${page.value * 25}`,
    {},
    false,
  )) as any;
  pages.value = newMods.total_hits;
  mods.value.hits.push(...newMods.hits);
  modsStatus.value = "success";
};

const versions = reactive<{ [key: string]: any[] }>({});

const getVersions = async (projectId: string) => {
  if (!versions[projectId]) {
    const allVersions = (await useBaseFetch(`project/${projectId}/version`, {}, false)) as any;

    if (props.isserver === false && props.type !== "modpack") {
      versions[projectId] = allVersions
        .filter((x: any) => x.loaders.includes(data.value?.loader?.toLocaleLowerCase()))
        .filter((x: any) => x.game_versions.includes(data.value?.mc_version))
        .map((x: any) => x.version_number);
    } else {
      versions[projectId] = allVersions.map((x: any) => x.version_number);
    }
  }
  return versions[projectId];
};

const selectedVersions = reactive<{ [key: string]: string }>({});

const expandedMods = reactive<{ [key: string]: { expanded: boolean; versions: any[] } }>({});

const toggleMod = async (modId: string) => {
  if (!expandedMods[modId]) {
    expandedMods[modId] = { expanded: false, versions: [] };
  }
  expandedMods[modId].expanded = !expandedMods[modId].expanded;
  if (expandedMods[modId].expanded && expandedMods[modId].versions.length === 0) {
    expandedMods[modId].versions = await getVersions(modId);
    // Select the first version by default
    if (expandedMods[modId].versions.length > 0) {
      selectedVersions[modId] = expandedMods[modId].versions[0];
    }
  }
};

const loadMore = async () => {
  page.value++;
  await loadMods();
};

const { reset } = useInfiniteScroll(scrollContainer, async () => {
  if (page.value <= pages.value) {
    await loadMore();
    console.log("loading more");
    console.log(page.value);
    console.log(pages.value);
  }
});

const resetList = () => {
  mods.value.hits = [];
  Object.keys(expandedMods).forEach((key) => delete expandedMods[key]);
  Object.keys(selectedVersions).forEach((key) => delete selectedVersions[key]);
  page.value = 0;
  loadMods();
  reset();
};

onMounted(async () => {
  await loadMods();
});
</script>
