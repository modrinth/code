<template>
  <Modal ref="editModal" header="">
    <div class="h-[500px]">
      <UiServersPyroModal @modal="editModal.hide()" header="Edit project">
        <UiServersProjectSelect type="modpack" :server="false" @select="createServer" />
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
          <Button icon-only @click="createServer('Vanilla')">
            <UiServersLoaderIcon loader="Vanilla" class="[&&]:size-10" />
          </Button>
          <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Vanilla</h1>
        </div>
        <div class="flex items-center gap-2">
          <Button icon-only @click="createServer('Fabric')">
            <UiServersLoaderIcon loader="Fabric" class="[&&]:size-10" />
          </Button>
          <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Fabric</h1>
        </div>
        <div class="flex items-center gap-2">
          <Button icon-only @click="createServer('Quilt')">
            <UiServersLoaderIcon loader="Quilt" class="[&&]:size-10" />
          </Button>
          <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Quilt</h1>
        </div>
        <div class="flex items-center gap-2">
          <Button icon-only @click="createServer('Forge')">
            <UiServersLoaderIcon loader="Forge" class="[&&]:size-10" />
          </Button>
          <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Forge</h1>
        </div>
        <div class="flex items-center gap-2">
          <Button icon-only @click="createServer('Neoforge')">
            <UiServersLoaderIcon loader="Neoforge" class="[&&]:size-10" />
          </Button>
          <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Neoforge</h1>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { DropdownSelect, Button, Modal } from "@modrinth/ui";
import { ChevronRightIcon, EditIcon, XIcon } from "@modrinth/assets";

const route = useNativeRoute();
const router = useRouter();
const config = useRuntimeConfig();

const editModal = ref();

const createServer = async (loaderorprojectid: any, version_number?: string) => {
  if (!version_number) {
    router.push(`/servers/new/${loaderorprojectid}`);
  } else {
    const versions = (await useBaseFetch(
      `project/${loaderorprojectid.project_id}/version`,
      {},
      false,
      config.public.prodOverride?.toLocaleLowerCase() === "true",
    )) as any[];
    const version_id = versions.find((x: any) => x.version_number === version_number)?.id;
    router.push(`/servers/new/${loaderorprojectid.project_id}?version=${version_id}`);
  }
};
</script>
