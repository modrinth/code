<template>
  <div class="h-full w-full">
    <div v-if="data && versions" class="flex h-full w-full flex-col gap-6 p-8">
      <div class="flex justify-between gap-2 rounded-xl bg-bg-raised p-4">
        <div class="flex gap-2">
          <UiAvatar :src="data.project.icon_url" size="96px" />
          <div class="flex flex-col gap-1">
            <h1 class="m-0 text-2xl font-extrabold leading-none text-contrast">
              {{ data.project.title }}
              <span class="rounded-full bg-bg-green p-1 px-2 text-sm font-semibold text-brand">
                v{{ currentVersion?.version_number }}
              </span>
            </h1>
            <span class="text-md font-semibold text-secondary">
              {{ data.project.description }}
            </span>
          </div>
        </div>
        <div>
          <Button icon-only>
            <EditIcon />
          </Button>
          <Button icon-only>
            <DownloadIcon />
          </Button>
        </div>
      </div>
    </div>
    <UiServersPyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
import { DropdownSelect, Button } from "@modrinth/ui";
import { DownloadIcon, EditIcon } from "@modrinth/assets";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const data = ref();
const versions = ref();
const options = ref();
const version = ref();
const currentVersion = ref();

const updateData = async () => {
  await serverStore.fetchServerData(serverId);
  const d = await serverStore.getServerData(serverId);
  data.value = d;
  const v = await useBaseFetch(`project/${d?.upstream?.project_id}/version`);
  versions.value = v;
  options.value = v.map((x) => x.version_number);
  currentVersion.value = await useBaseFetch(`version/${d?.upstream?.version_id}`);
};
updateData();
</script>
