<template>
  <div class="relative h-full w-full overflow-y-auto">
    <div v-if="data" class="flex h-full w-full flex-col justify-between gap-6 px-8 py-4">
      <h2 class="m-0 text-3xl font-bold">Network</h2>
      <div class="flex h-full flex-col gap-2">
        <div class="card flex flex-col gap-4">
          <label for="username-field" class="flex flex-col gap-2">
            <span class="text-lg font-bold text-contrast">Subdomain</span>
            <span>
              Change the subdomain of your server. Your friends can use this subdomain to connect to
              your server.
            </span>
          </label>
          <div class="flex w-full items-center gap-2 md:w-[60%]">
            <input v-model="serverSubdomain" class="h-[50%] w-[63%]" @keyup.enter="saveNetwork" />
            .modrinth.gg
          </div>
        </div>
        <div class="card flex flex-col gap-4">
          <div class="flex w-full flex-row items-center justify-between">
            <label for="username-field" class="flex flex-col gap-2">
              <span class="text-lg font-bold text-contrast">Allocations</span>
              <span>
                Configure ports for internet-facing features like map viewers or voice chat mods.
              </span>
            </label>

            <ButtonStyled type="standard" color="brand">
              <button>
                <PlusIcon />
                <span>New Allocation</span>
              </button>
            </ButtonStyled>
          </div>

          <div class="flex w-full flex-col overflow-hidden rounded-xl">
            <!-- allocation row -->
            <div class="flex flex-row items-center justify-between bg-bg px-4 py-4">
              <div class="flex flex-row items-center gap-4">
                <span class="text-sm font-bold uppercase tracking-wide">Primary Allocation</span>
                <div class="font-[family-name:var(--mono-font)]">
                  <CopyCode :text="`${serverIP}:${serverPrimaryPort}`" />
                </div>
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
        :is-updating="isUpdating"
        :save="saveNetwork"
        :reset="resetNetwork"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { PlusIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";
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

const hasUnsavedChanges = computed(() => serverSubdomain.value !== data?.value?.net?.domain);

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
    await serverStore.changeSubdomain(serverId, serverSubdomain.value);
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
};
</script>
