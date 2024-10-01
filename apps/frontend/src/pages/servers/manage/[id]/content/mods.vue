<template>
  <Modal ref="addModModal" header="Add mod">
    <div class="h-[500px]">
      <UiServersModSelect />
    </div>
  </Modal>
  <div class="flex h-full w-full flex-col">
    <div
      class="flex items-center justify-between gap-2 border-0 border-b border-solid border-bg-raised p-3"
    >
      <h2 class="m-0 text-2xl font-bold text-contrast">Mods</h2>
      <Button icon-only transparent @click="addModModal.show()">
        <PlusIcon class="h-10 w-10 text-contrast" />
      </Button>
    </div>
    <div
      class="flex h-full w-full flex-col overflow-y-scroll"
      v-if="data && status == 'success' && mods"
    >
      <UiServersContentItem
        v-for="mod in mods"
        :key="mod.name"
        :name="mod.name"
        :filename="mod.filename"
        :version_number="mod.version_number"
        :disabled="mod.disabled"
        :icon="mod.icon_url"
        :project="mod.project_id"
        :version="mod.version_id"
        :icon_url="mod.icon_url"
      />
    </div>
    <UiServersPyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
import { PlusIcon } from "@modrinth/assets";
import { Button, Modal } from "@modrinth/ui";

interface Mod {
  name: string;
  filename: string;
  project_id: string;
  version_id: string;
  version_number: string;
  icon_url: string;
  disabled: boolean;
}

const serverStore = useServerStore();
const route = useNativeRoute();
const serverId = route.params.id as string;

const addModModal = ref();

const { data, status } = await useLazyAsyncData("modsData", async () => {
  await serverStore.fetchServerData(serverId);
  return serverStore.getServerData(serverId);
});

const { data: mods } = await useLazyAsyncData("modsData", async () => {
  return (await serverStore.getMods(serverId)) as Mod[];
});

computed(() => {
  mods.value?.sort((a: any, b: any) => {
    if (a.name < b.name) return -1;
    if (a.name > b.name) return 1;
    return 0;
  });
});
</script>
