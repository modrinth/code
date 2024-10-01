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
        placeholder="Search filters..."
        autocomplete="off"
        @keyup.enter="refreshNuxtData('modsResults')"
      />
    </div>
    <div class="flex h-[93%] w-full flex-col">
      <div class="flex h-full w-full flex-col gap-2 overflow-y-scroll pt-2" v-if="mods">
        <div v-for="mod in mods.hits" :key="mod.title" class="rounded-lg p-2 hover:bg-divider-dark">
          <div class="flex cursor-pointer gap-2" @click="toggleMod(mod.project_id)">
            <UiAvatar :src="mod.icon_url" size="sm" circle />
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
          <div v-if="expandedMods[mod.project_id as keyof typeof expandedMods]" class="mt-2">
            <DropdownSelect
              id="version-select"
              v-model="selectedVersion"
              name="version-select"
              :options="mod.versions"
              placeholder="Select version..."
            />
          </div>
        </div>
      </div>
      <UiServersPyroLoading v-else />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from "vue";
import { SearchIcon } from "@modrinth/assets";
import { Button, DropdownSelect } from "@modrinth/ui";

const serverStore = useServerStore();
const route = useNativeRoute();
const serverId = route.params.id as string;

const { data, status } = await useLazyAsyncData("modsData", async () => {
  await serverStore.fetchServerData(serverId);
  return serverStore.getServerData(serverId);
});

const queryFilter = ref("");
const facets = ref<any>([]);
facets.value.push(`["categories:${data.value?.loader?.toLocaleLowerCase()}"]`);
facets.value.push(`["versions:${data.value?.mc_version}"]`);
if (data.value?.loader) {
  facets.value.push('["project_type:mod"]');
}

const buildFacetString = (facets: string[]) => {
  return "[" + facets.map((facet) => `${facet}`).join(",") + "]";
};

const { data: mods } = await useLazyAsyncData("modsResults", async () => {
  return await useBaseFetch(
    `search?query=${queryFilter.value}&facets=${buildFacetString(facets.value)}&index=relevance&limit=20`,
  );
});

const selectedVersion = ref("");

const expandedMods = reactive<{ [key: string]: boolean }>({});

const toggleMod = (modId: string) => {
  expandedMods[modId] = !expandedMods[modId];
  selectedVersion.value = "";
};
</script>
