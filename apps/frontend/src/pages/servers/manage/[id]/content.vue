<template>
  <div>
    <div v-if="data && status === 'success'">
      <div class="flex flex-col gap-4">
        <div
          class="relative grid w-full grid-cols-2 items-center gap-4 overflow-hidden rounded-2xl bg-bg-raised p-8"
        >
          <div class="flex flex-col">
            <div class="text-2xl font-extrabold text-[var(--color-contrast)]">Current instance</div>
            <p>
              {{ formatMessage(messages.currentInstanceDescription) }}
            </p>
          </div>
          <div class="flex flex-row items-center">
            <div class="z-50 flex w-full gap-4 rounded-2xl bg-button-bg p-3">
              <img
                v-if="data && data.project"
                alt="Server Icon"
                class="h-full w-full max-w-20 rounded-2xl object-cover align-middle"
                :src="data.project.icon_url"
                loading="lazy"
                decoding="async"
              />
              <div class="flex w-full justify-between">
                <div v-if="data" class="flex flex-col">
                  <h1 class="m-0 text-xl font-bold text-[var(--color-contrast)]">
                    {{ data.project?.title ?? "" }}
                  </h1>
                  <div class="text-sm">
                    {{ data.project?.description ?? "" }}
                  </div>
                </div>
                <div class="flex flex-col gap-1">
                  <button class="rounded-xl border-2 border-solid border-[#424956] bg-bg p-2 px-4">
                    <EditIcon />
                  </button>
                  <button class="rounded-xl border-2 border-solid border-[#424956] bg-bg p-2 px-4">
                    <ImportIcon />
                  </button>
                </div>
              </div>
            </div>
            <div class="-m-1 flex h-fit flex-col justify-center gap-1 rounded-r-lg bg-bg p-2 pl-3">
              <div class="flex flex-col">
                <div class="text-sm font-bold text-[var(--color-contrast)]">Current</div>
                <div class="text-xs font-semibold">v1.0.0</div>
              </div>
              <div class="flex flex-col">
                <div class="text-sm font-bold text-[var(--color-contrast)]">Latest</div>
                <div class="text-xs font-semibold">v1.0.1b</div>
              </div>
            </div>
          </div>
        </div>
        <div
          class="relative w-full items-center gap-4 overflow-hidden rounded-2xl bg-bg-raised p-8"
        >
          <div class="flex w-full flex-col gap-4 rounded-xl bg-bg p-2">
            <div class="flex items-center justify-between gap-4">
              <div class="flex items-center gap-2 text-xl font-bold text-[var(--color-contrast)]">
                <BoxIcon class="h-6 w-6" /> Server Content
              </div>
              <div class="mr-2 flex items-center gap-4">
                <div
                  class="flex items-center gap-2 text-sm font-semibold text-[var(--color-text-base)]"
                >
                  <FileIcon class="h-6 w-6" />
                  Mods
                  <div class="h-1 w-1 rounded-full bg-[#424956]"></div>
                  <span class="font-medium text-[var(--color-text-secondary)]"> 47 </span>
                </div>

                <div
                  class="flex items-center gap-2 text-sm font-semibold text-[var(--color-text-base)]"
                >
                  <LoaderIcon v-if="data.loader" :loader="data.loader" class="" />
                  Forge
                  <div class="h-1 w-1 rounded-full bg-[#424956]"></div>
                  <span class="font-medium text-[var(--color-text-secondary)]">
                    {{ data.loader_version }}
                  </span>
                </div>
              </div>
            </div>
            <div class="relative mb-4 w-full text-sm">
              <label class="sr-only" for="search">Search</label>
              <SearchIcon
                class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2"
                aria-hidden="true"
              />
              <input
                id="search"
                class="w-full border-[1px] border-solid border-[#33363D] pl-9"
                type="search"
                name="search"
                autocomplete="off"
                placeholder="Search servers..."
              />
            </div>
            <div></div>
          </div>
        </div>
      </div>
    </div>
    <UiServersPyroError
      v-else-if="status === 'error'"
      title="Error Accessing Server"
      message="Dont worry, your server is safe. We just can't connect to it right now."
    />
    <PyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
import { Button } from "@modrinth/ui";
import { EditIcon, ImportIcon, BoxIcon, SearchIcon, FileIcon } from "@modrinth/assets";
import LoaderIcon from "~/components/ui/servers/LoaderIcon.vue";
import { useServerStore } from "~/stores/servers";
import PyroLoading from "~/components/ui/servers/PyroLoading.vue";

const { formatMessage } = useVIntl();
const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();
const auth = await useAuth();

await serverStore.fetchServerData(serverId);
const { data, status } = await useLazyAsyncData("contentServerData", async () =>
  serverStore.getServerData(serverId),
);

const { data: mods, status: modsStatus } = await useLazyAsyncData("serverProps", async () => {
  await usePyroFetch<string>(auth.value.token, `servers/${serverId}/mods`);
});

console.log(await mods);

const messages = defineMessage({
  currentInstanceLabel: {
    id: "servers.manage.content.currentInstanceLabel",
    defaultMessage: "Pack and Loader",
  },
  currentInstanceDescription: {
    id: "servers.manage.content.currentInstanceDescription",
    defaultMessage: "Manage any additional content on your server like mods and modpacks.",
  },
});
</script>
