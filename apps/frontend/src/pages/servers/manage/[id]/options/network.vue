<template>
  <div>
    <div v-if="data && status === 'success'">
      <section class="card">
        <div class="flex flex-col gap-6">
          <h2 class="text-3xl font-bold">Network</h2>
          <div class="h-[2px] w-full bg-divider"></div>
          <div class="ml-1 flex items-center justify-between">
            <label for="username-field" class="flex flex-col gap-2">
              <span class="text-lg font-bold text-white">Subdomain</span>
              <span> Change the subdomain to connect to your server </span>
            </label>
            <div class="flex items-center gap-2">
              <input
                v-if="subdomain"
                v-model="subdomain"
                :placeholder="subdomain"
                class="h-[50%] w-[63%]"
              />
              .{{ data.net.domain.split(".").slice(1).join(".") }}
            </div>
          </div>
          <button type="submit" class="btn btn-primary" @click="" :disabled="isUpdating">
            {{ isUpdating ? "Saving..." : "Save" }}
          </button>
        </div>
      </section>
    </div>
    <UiServersPyroLoading v-else-if="status === 'pending'" />
  </div>
</template>

<script setup lang="ts">
import { useServerStore } from "~/stores/servers.ts";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const isUpdating = ref(false);

await serverStore.fetchServerData(serverId);
const { data, status } = await useLazyAsyncData(
  "infoServerData",
  async () => await serverStore.getServerData(serverId),
);

const subdomain = ref<string>(data?.value?.net?.domain.split(".")[0] ?? "");
</script>
