<template>
  <Modal ref="addModModal" header="">
    <div class="h-[500px]">
      <UiServersPyroModal @modal="addModModal.hide()" header="Add Datapack" data="data">
        <UiServersProjectSelect type="datapack" />
      </UiServersPyroModal>
    </div>
  </Modal>
  <div class="flex h-full w-full flex-col">
    <div
      class="flex items-center justify-between gap-2 border-0 border-b border-solid border-bg-raised p-3"
    >
      <h2 class="m-0 text-2xl font-bold text-contrast">DataPacks</h2>
      <Button icon-only transparent @click="addModModal.show()">
        <PlusIcon class="h-10 w-10 text-contrast" />
      </Button>
    </div>
    <div
      class="flex h-full w-full flex-col overflow-y-scroll"
      v-if="data && status == 'success'"
    ></div>
    <UiServersPyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
import { PlusIcon } from "@modrinth/assets";
import { Button, Modal } from "@modrinth/ui";

const serverStore = useServerStore();
const route = useNativeRoute();
const serverId = route.params.id as string;

const addModModal = ref();

const { data, status } = await useLazyAsyncData("content-datapacks-data", async () => {
  await serverStore.fetchServerData(serverId);
  return serverStore.getServerData(serverId);
});
</script>
