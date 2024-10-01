<template>
  <Modal ref="editModal" header="Edit project">
    <div class="flex flex-col items-center justify-between gap-4 p-6">
      <div class="w-[10rem] rounded-xl bg-bg p-8">Loader</div>
      or
      <div class="w-[10rem] rounded-xl bg-bg p-8">Modpack</div>
    </div>
  </Modal>
  <div class="h-full w-full">
    <div v-if="data && versions" class="flex h-full w-full flex-col gap-6 p-8">
      <div class="flex w-full gap-2">
        <div class="flex w-full justify-between gap-2 rounded-xl bg-bg-raised p-4">
          <div class="flex gap-4">
            <UiAvatar :src="data.project.icon_url" size="120px" />
            <div class="flex flex-col justify-between">
              <div class="flex flex-col gap-2">
                <h1 class="m-0 text-2xl font-extrabold leading-none text-contrast">
                  {{ data.project.title }}
                  <span class="rounded-full bg-bg-green p-1 px-2 text-sm font-semibold text-brand">
                    v{{ currentVersion?.version_number }}
                  </span>
                </h1>
                <span class="text-md font-semibold text-secondary">
                  {{
                    data.project.description.length > 150
                      ? data.project.description.substring(0, 150) + "..."
                      : data.project.description
                  }}
                </span>
              </div>
              <div class="flex w-[16rem] items-center gap-2">
                <DropdownSelect
                  v-if="versions.length > 0"
                  v-model="version"
                  :options="options"
                  placeholder="Change version"
                />
                <Button icon-only>
                  <DownloadIcon />
                </Button>
              </div>
            </div>
          </div>
        </div>
        <div
          class="flex flex-col items-center justify-center rounded-xl bg-bg-raised p-2 hover:bg-button-bg"
          @click="editModal.show()"
        >
          <EditIcon class="h-6 w-6 text-contrast" />
        </div>
      </div>
      <div class="flex items-center gap-2 bg-bg-raised p-2">
        <Button icon-only>
          <img alt="" class="h-20 w-20 rounded-full" src="~/assets/images/games/minecraft.png" />
        </Button>
        <Button icon-only>
          <UiServersLoaderIcon loader="Fabric" width="810" height="540" />
        </Button>
        <Button icon-only>
          <UiServersLoaderIcon loader="Quilt" />
        </Button>
        <Button icon-only>
          <UiServersLoaderIcon loader="Forge" />
        </Button>
        <Button icon-only>
          <UiServersLoaderIcon loader="Neoforge" />
        </Button>
      </div>
    </div>
    <UiServersPyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
import { DropdownSelect, Button, Modal } from "@modrinth/ui";
import { DownloadIcon, EditIcon } from "@modrinth/assets";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const editModal = ref();

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
