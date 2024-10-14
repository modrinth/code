<template>
  <Modal ref="newAllocationModal" header="">
    <UiServersPyroModal header="New allocation" :data="data" @modal="newAllocationModal?.hide()">
      <div class="flex flex-col gap-2">
        <div class="font-semibold text-contrast">Name<span class="text-red-500">*</span></div>
        <input
          v-model="newAllocationName"
          type="text"
          class="bg-bg-input w-full rounded-lg p-4"
          placeholder="e.g. Secondary allocation"
        />
      </div>
      <div class="mb-1 mt-4 flex justify-end gap-4">
        <Button transparent @click="newAllocationModal?.hide()"> Cancel </Button>
        <Button color="primary" @click="addNewAllocation"> <PlusIcon /> Create allocation </Button>
      </div>
    </UiServersPyroModal>
  </Modal>

  <Modal ref="editAllocationModal" header="">
    <UiServersPyroModal header="Edit allocation" :data="data" @modal="editAllocationModal?.hide()">
      <div class="flex flex-col gap-2">
        <div class="font-semibold text-contrast">Name<span class="text-red-500">*</span></div>
        <input
          v-model="newAllocationName"
          type="text"
          class="bg-bg-input w-full rounded-lg p-4"
          placeholder="e.g. Secondary allocation"
        />
      </div>
      <div class="mb-1 mt-4 flex justify-end gap-4">
        <Button transparent @click="editAllocationModal?.hide()"> Cancel </Button>
        <Button color="primary" @click="editAllocation"> <SaveIcon /> Update Allocation </Button>
      </div>
    </UiServersPyroModal>
  </Modal>

  <div class="relative h-full w-full overflow-y-auto">
    <div v-if="data" class="flex h-full w-full flex-col justify-between gap-4">
      <div class="flex h-full flex-col">
        <!-- Subdomain section -->
        <div class="card flex flex-col gap-4">
          <label for="username-field" class="flex flex-col gap-2">
            <span class="text-lg font-bold text-contrast">Custom URL</span>
            <span> Your friends can connect to your server using this URL. </span>
          </label>
          <div class="flex w-full items-center gap-2 md:w-[60%]">
            <input v-model="serverSubdomain" class="h-[50%] w-[63%]" @keyup.enter="saveNetwork" />
            .modrinth.gg
          </div>
        </div>

        <!-- Allocations section -->
        <div class="card flex flex-col gap-4">
          <div class="flex w-full flex-row items-center justify-between">
            <label for="username-field" class="flex flex-col gap-2">
              <span class="text-lg font-bold text-contrast">Allocations</span>
              <span>
                Configure additional ports for internet-facing features like map viewers or voice
                chat mods.
              </span>
            </label>

            <ButtonStyled type="standard" color="brand" @click="newAllocationModal.show()">
              <button>
                <PlusIcon />
                <span>New Allocation</span>
              </button>
            </ButtonStyled>
          </div>

          <div class="flex w-full flex-col overflow-hidden rounded-xl bg-table-alternateRow p-4">
            <!-- Primary allocation -->
            <div class="flex flex-row items-center justify-between">
              <span class="text-md font-bold capitalize tracking-wide text-contrast">
                Primary Allocation
              </span>

              <UiCopyCode :text="`${serverIP}:${serverPrimaryPort}`" />
            </div>
          </div>

          <div
            v-if="allocations?.[0]"
            class="flex w-full flex-col gap-4 overflow-hidden rounded-xl bg-table-alternateRow p-4"
          >
            <div
              v-for="allocation in allocations"
              :key="allocation.port"
              class="border-border flex flex-row items-center justify-between"
            >
              <div class="flex flex-row items-center gap-4">
                <VersionIcon class="h-7 w-7 rotate-90" />
                <div class="flex w-[20rem] flex-row items-center justify-between">
                  <div class="flex flex-col gap-1">
                    <span class="text-md font-bold capitalize tracking-wide text-contrast">
                      {{ allocation.name }}
                    </span>
                    <span class="text-xs uppercase text-secondary">name</span>
                  </div>
                  <div class="flex flex-col gap-1">
                    <span class="text-md w-10 font-bold capitalize tracking-wide text-contrast">
                      {{ allocation.port }}
                    </span>
                    <span class="text-xs uppercase text-secondary">port</span>
                  </div>
                </div>
              </div>

              <div class="flex flex-row items-center gap-2">
                <Button icon-only @click="showEditAllocation(allocation.port)">
                  <EditIcon />
                </Button>
                <Button icon-only color="danger" @click="removeAllocation(allocation.port)">
                  <TrashIcon />
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
    <UiServersPyroLoading v-else />
    <div class="absolute bottom-[2.5%] left-[2.5%] z-10 w-[95%]">
      <UiServersSaveBanner
        v-if="hasUnsavedChanges"
        :server="props.server"
        :is-updating="isUpdating"
        :save="saveNetwork"
        :reset="resetNetwork"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
// currently broken, will fix

import { PlusIcon, TrashIcon, EditIcon, VersionIcon, SaveIcon } from "@modrinth/assets";
import { ButtonStyled, Modal, Button } from "@modrinth/ui";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
}>();

const isUpdating = ref(false);
const data = computed(() => props.server.general);

const serverIP = ref(data?.value?.net?.ip ?? "");
const serverSubdomain = ref(data?.value?.net?.domain ?? "");
const serverPrimaryPort = ref(data?.value?.net?.port ?? 0);

const network = computed(() => props.server.network);
const allocations = ref(network.value?.allocations);

const newAllocationModal = ref();
const editAllocationModal = ref();
const newAllocationName = ref("");
const newAllocationPort = ref(0);

const hasUnsavedChanges = computed(() => serverSubdomain.value !== data?.value?.net?.domain);

const addNewAllocation = async () => {
  if (!newAllocationName.value) return;

  try {
    await props.server.network?.reserveAllocation(newAllocationName.value);

    await props.server.refresh();

    newAllocationModal.value.hide();
    newAllocationName.value = "";
  } catch (error) {
    console.error("Failed to reserve new allocation:", error);
  }
};

const showEditAllocation = (port: number) => {
  newAllocationPort.value = port;
  editAllocationModal.value.show();
};

const editAllocation = async () => {
  if (!newAllocationName.value) return;

  try {
    await props.server.network?.updateAllocation(newAllocationPort.value, newAllocationName.value);

    await props.server.refresh();

    editAllocationModal.value.hide();
    newAllocationName.value = "";
  } catch (error) {
    console.error("Failed to reserve new allocation:", error);
  }
};

const removeAllocation = async (port: number) => {
  await props.server.network?.deleteAllocation(port);
  await props.server.refresh();
};

const saveNetwork = async () => {
  try {
    isUpdating.value = true;
    const available = await props.server.network?.checkSubdomainAvailability(serverSubdomain.value);
    if (!available) {
      addNotification({
        group: "serverOptions",
        type: "error",
        title: "Subdomain not available",
        text: "The subdomain you entered is already in use.",
      });
      return;
    }
    if (serverSubdomain.value !== data?.value?.net?.domain) {
      await props.server.network?.changeSubdomain(serverSubdomain.value);
    }
    if (serverPrimaryPort.value !== data?.value?.net?.port) {
      await props.server.network?.updateAllocation(
        serverPrimaryPort.value,
        newAllocationName.value,
      );
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
    await props.server.refresh();
    addNotification({
      group: "serverOptions",
      type: "success",
      title: "Server settings updated",
      text: "Your server settings were successfully changed.",
    });
  } catch (error) {
    console.error(error);
    addNotification({
      group: "serverOptions",
      type: "error",
      title: "Failed to update server settings",
      text: "An error occurred while attempting to update your server settings.",
    });
  } finally {
    isUpdating.value = false;
  }
};

const resetNetwork = () => {
  serverSubdomain.value = data?.value?.net?.domain ?? "";
  allocations.value = network.value?.allocations ?? [];
};
</script>
