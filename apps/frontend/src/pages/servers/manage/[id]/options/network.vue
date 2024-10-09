<template>
  <Modal ref="newAllocationModal" header="New Allocation">
    <div class="flex flex-col gap-4">
      <label for="allocation-name" class="flex flex-col gap-2">
        <span class="text-lg font-bold text-contrast">Allocation Name</span>
        <input id="allocation-name" v-model="newAllocationName" class="w-full" />
      </label>
    </div>
    <div class="button-row">
      <Button @click="newAllocationModal.hide()"> Cancel </Button>
      <Button color="primary" @click="addNewAllocation"> Create </Button>
    </div>
  </Modal>
  <div class="relative h-full w-full overflow-y-auto">
    <div v-if="data" class="flex h-full w-full flex-col justify-between gap-4 px-4">
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
                Configure ports for internet-facing features like map viewers or voice chat mods.
              </span>
            </label>

            <ButtonStyled type="standard" color="brand" @click="openNewAllocationModal">
              <button>
                <PlusIcon />
                <span>New Allocation</span>
              </button>
            </ButtonStyled>
          </div>

          <div class="flex w-full flex-col overflow-hidden rounded-xl">
            <!-- Primary allocation -->
            <div class="flex flex-row items-center justify-between bg-bg px-4 py-4">
              <div class="flex flex-row items-center gap-4">
                <span class="text-sm font-bold uppercase tracking-wide">Primary Allocation</span>
                <div class="font-[family-name:var(--mono-font)]">
                  <CopyCode :text="`${serverIP}:${serverPrimaryPort}`" />
                </div>
              </div>
            </div>

            <!-- Additional allocations -->
            <div
              v-for="allocation in allocations"
              :key="allocation.port"
              class="border-border flex flex-row items-center justify-between border-t bg-bg px-4 py-4"
            >
              <div class="flex flex-row items-center gap-4">
                <span class="text-sm font-bold uppercase tracking-wide">{{ allocation.name }}</span>
                <div class="font-[family-name:var(--mono-font)]">
                  <CopyCode :text="`${serverIP}:${allocation.port}`" />
                </div>
              </div>
              <ButtonStyled type="standard" @click="removeAllocation(allocation.port)">
                <button>
                  <TrashIcon />
                  <span>Remove</span>
                </button>
              </ButtonStyled>
            </div>
          </div>
        </div>
      </div>
    </div>
    <UiServersPyroLoading v-else />
    <div class="absolute bottom-[2.5%] left-[2.5%] z-10 w-[95%]">
      <UiServersSaveBanner
        v-if="hasUnsavedChanges"
        :is-updating="isUpdating"
        :save="saveNetwork"
        :reset="resetNetwork"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { PlusIcon, TrashIcon } from "@modrinth/assets";
import { ButtonStyled, Modal } from "@modrinth/ui";
import CopyCode from "~/components/ui/CopyCode.vue";
import { useServerStore } from "~/stores/servers.ts";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();
const app = useNuxtApp();

const isUpdating = ref(false);
const data = computed(() => serverStore.serverData[serverId]);

const serverIP = ref(data?.value?.net?.ip ?? "");
const serverSubdomain = ref(data?.value?.net?.domain ?? "");
const serverPrimaryPort = ref(data?.value?.net?.port ?? 0);

const allocations = ref(data?.value?.net?.allocations ?? []);
const newAllocationModal = ref();
const newAllocationName = ref("");

const hasUnsavedChanges = computed(() => serverSubdomain.value !== data?.value?.net?.domain);

const openNewAllocationModal = () => {
  newAllocationModal.value.show();
};

const addNewAllocation = async () => {
  if (!newAllocationName.value) return;

  try {
    const response = await serverStore.reserveAllocation(serverId, newAllocationName.value);
    const newPort = response.port;

    allocations.value.push({
      name: newAllocationName.value,
      port: newPort,
    });

    newAllocationModal.value.hide();
    newAllocationName.value = "";
  } catch (error) {
    console.error("Failed to reserve new allocation:", error);
  }
};

const removeAllocation = (port: number) => {
  allocations.value = allocations.value.filter((allocation) => allocation.port !== port);
};

const saveNetwork = async () => {
  try {
    isUpdating.value = true;
    const available = (await serverStore.checkSubdomainAvailability(serverSubdomain.value)) as {
      available: boolean;
    };
    if (!available.available) {
      // @ts-ignore
      app.$notify({
        group: "serverOptions",
        type: "error",
        title: "Subdomain not available",
        text: "The subdomain you entered is already in use.",
      });
      return;
    }
    if (serverSubdomain.value !== data?.value?.net?.domain) {
      await serverStore.changeSubdomain(serverId, serverSubdomain.value);
    }
    if (serverPrimaryPort.value !== data?.value?.net?.port) {
      await serverStore.updateAllocation(
        serverId,
        serverPrimaryPort.value,
        newAllocationName.value,
      );
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
    // @ts-ignore
    app.$notify({
      group: "serverOptions",
      type: "success",
      title: "Server settings updated",
      text: "Your server settings were successfully changed.",
    });
  } catch (error) {
    console.error(error);
    // @ts-ignore
    app.$notify({
      group: "serverOptions",
      type: "error",
      title: "Failed to update server settings",
      text: "An error occurred while attempting to update your server settings.",
    });
  } finally {
    isUpdating.value = false;
    resetNetwork();
    await serverStore.fetchServerData(serverId);
    resetNetwork();
  }
};

const resetNetwork = () => {
  serverSubdomain.value = data?.value?.net?.domain ?? "";
  allocations.value = data?.value?.net?.allocations ?? [];
};
</script>
