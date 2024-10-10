<template>
  <Modal ref="editModal" header="">
    <div class="h-[500px]">
      <UiServersPyroModal header="Edit project" @modal="editModal.hide()">
        <UiServersProjectSelect type="modpack" :server="false" @select="createServer" />
      </UiServersPyroModal>
    </div>
  </Modal>

  <Modal ref="versionSelectModal" header="">
    <div class="h-[500px]">
      <UiServersPyroModal header="Select Versions" @modal="versionSelectModal.hide()">
        <div class="flex items-center gap-4">
          <DropdownSelect
            v-model="gameVersion"
            name="game-version"
            :options="
              tags.gameVersions.filter((x) => x.version_type === 'release').map((x) => x.version)
            "
            placeholder="Select game version..."
            @change="getLoaderVersions(loader)"
          />
          <DropdownSelect
            v-if="gameVersion"
            v-model="loaderVersion"
            name="loader-version"
            :options="loaderVersions"
            placeholder="Select loader version..."
          />
          <Button icon-only @click="createServer">
            <ChevronRightIcon />
          </Button>
        </div>
      </UiServersPyroModal>
    </div>
  </Modal>

  <div class="flex h-full w-full justify-center">
    <div
      class="border-sold border-1 flex w-[1200px] flex-col gap-6 rounded-xl border-bg-raised p-8"
    >
      <div
        class="flex w-full justify-center gap-2 rounded-xl bg-button-bg p-4 text-xl font-bold text-contrast hover:bg-button-bgActive"
        @click="editModal.show()"
      >
        ModPack
      </div>
      <div class="flex w-full items-center justify-between rounded-xl bg-bg-raised p-2 pr-4">
        <div class="flex items-center gap-2">
          <Button icon-only @click="selectVersions('Vanilla')">
            <UiServersLoaderIcon loader="Vanilla" class="[&&]:size-10" />
          </Button>
          <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Vanilla</h1>
        </div>
        <div class="flex items-center gap-2">
          <Button icon-only @click="selectVersions('Fabric')">
            <UiServersLoaderIcon loader="Fabric" class="[&&]:size-10" />
          </Button>
          <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Fabric</h1>
        </div>
        <div class="flex items-center gap-2">
          <Button icon-only @click="selectVersions('Quilt')">
            <UiServersLoaderIcon loader="Quilt" class="[&&]:size-10" />
          </Button>
          <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Quilt</h1>
        </div>
        <div class="flex items-center gap-2">
          <Button icon-only @click="selectVersions('Forge')">
            <UiServersLoaderIcon loader="Forge" class="[&&]:size-10" />
          </Button>
          <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Forge</h1>
        </div>
        <div class="flex items-center gap-2">
          <Button icon-only @click="selectVersions('Neoforge')">
            <UiServersLoaderIcon loader="Neoforge" class="[&&]:size-10" />
          </Button>
          <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Neoforge</h1>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button, Modal, DropdownSelect } from "@modrinth/ui";
import { ChevronRightIcon } from "@modrinth/assets";

const router = useRouter();
const prodOverride = await PyroAuthOverride();
const tags = useTags();

const editModal = ref();
const versionSelectModal = ref();

const loader = ref();
const loaderVersion = ref();
const gameVersion = ref();
const loaderVersions = ref<string[]>([]);

const getLoaderVersions = async (loader: string) => {
  const url = `https://launcher-meta.modrinth.com/${loader.toLocaleLowerCase()}/v0/manifest.json`;
  const data = ref<any>();
  try {
    const res = await $fetch(url, {
      method: "GET",
      retry: false,
      headers: {
        "User-Agent": "Pyro/1.0 (https://pyro.host)",
        "Access-Control-Allow-Origin": "*",
      },
    });

    data.value = res;
  } catch (error) {
    console.error("Error fetching loader versions:", error);
  }

  const loaderVersions = data.value?.gameVersions
    .filter((x: any) => x.id === gameVersion.value)[0]
    ?.loaders.map((x: any) => x.id);
  console.log(loaderVersions);
  return loaderVersions;
};

const selectVersions = (inLoader: string) => {
  loader.value = inLoader;
  versionSelectModal.value.show();
};

const createServer = async (loaderorprojectid: any, versionNumber?: string) => {
  if (!versionNumber) {
    router.push(
      `/servers/new/${loaderorprojectid}?loader_version=${loaderVersion}&game_version=${gameVersion}`,
    );
  } else {
    const versions = (await useBaseFetch(
      `project/${loaderorprojectid.project_id}/version`,
      {},
      false,
      prodOverride,
    )) as any[];
    const versionId = versions.find((x: any) => x.version_number === versionNumber)?.id;
    router.push(`/servers/new/${loaderorprojectid.project_id}?version=${versionId}`);
  }
};
</script>
