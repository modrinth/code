<template>
  <NuxtLink
    class="contents"
    :to="status === 'suspended' ? '' : `/servers/manage/${props.server_id}`"
  >
    <div
      v-tooltip="
        status === 'suspended'
          ? suspension_reason === 'upgrading'
            ? 'This server is being transferred to a new node. It will be unavailable until this process finishes.'
            : 'This server has been suspended. Please visit your billing settings or contact Modrinth Support for more information.'
          : ''
      "
      class="bg-bg-raised flex cursor-pointer flex-row items-center overflow-x-hidden rounded-3xl p-4 transition-transform duration-100"
      :class="status === 'suspended' ? 'rounded-b-none! opacity-75' : 'active:scale-95'"
      data-pyro-server-listing
      :data-pyro-server-listing-id="server_id"
    >
      <UiServersServerIcon v-if="status !== 'suspended'" :image="image" />
      <div
        v-else
        class="bg-bg-secondary border-button-border bg-button-bg flex size-24 items-center justify-center rounded-xl border shadow-xs"
      >
        <LockIcon class="text-secondary size-20" />
      </div>
      <div class="ml-8 flex flex-col gap-2.5">
        <div class="flex flex-row items-center gap-2">
          <h2 class="text-contrast m-0 text-xl font-bold">{{ name }}</h2>
          <ChevronRightIcon />
        </div>

        <div
          v-if="projectData?.title"
          class="text-secondary m-0 flex flex-row items-center gap-2 text-sm font-medium"
        >
          <UiAvatar
            :src="iconUrl"
            no-shadow
            style="min-height: 20px; min-width: 20px; height: 20px; width: 20px"
            alt="Server Icon"
          />
          Using {{ projectData?.title || "Unknown" }}
        </div>
        <div v-else class="min-h-[20px]"></div>

        <div
          v-if="isConfiguring"
          class="text-brand flex min-w-0 items-center gap-2 truncate text-sm font-semibold"
        >
          <SparklesIcon class="size-5 shrink-0" /> New server
        </div>
        <UiServersServerInfoLabels
          v-else
          :server-data="{ game, mc_version, loader, loader_version, net }"
          :show-game-label="showGameLabel"
          :show-loader-label="showLoaderLabel"
          :linked="false"
          class="text-secondary pointer-events-none flex w-full flex-row flex-wrap items-center gap-4 *:hidden sm:flex-row sm:*:flex"
        />
      </div>
    </div>
    <div
      v-if="status === 'suspended' && suspension_reason === 'upgrading'"
      class="bg-bg-blue text-contrast relative -mt-4 flex w-full flex-row items-center gap-2 rounded-b-3xl p-4 text-sm font-bold"
    >
      <UiServersPanelSpinner />
      Your server's hardware is currently being upgraded and will be back online shortly.
    </div>
    <div
      v-else-if="status === 'suspended'"
      class="bg-bg-red text-contrast relative -mt-4 flex w-full flex-col gap-2 rounded-b-3xl p-4 text-sm font-bold"
    >
      <div class="flex flex-row gap-2">
        <UiServersIconsPanelErrorIcon class="size-5!" /> Your server has been suspended. Please
        update your billing information or contact Modrinth Support for more information.
      </div>
      <UiCopyCode :text="`${props.server_id}`" class="ml-auto" />
    </div>
  </NuxtLink>
</template>

<script setup lang="ts">
import { ChevronRightIcon, LockIcon, SparklesIcon } from "@modrinth/assets";
import type { Project, Server } from "@modrinth/utils";
import { useModrinthServers } from "~/composables/servers/modrinth-servers.ts";

const props = defineProps<Partial<Server>>();

if (props.server_id) {
  await useModrinthServers(props.server_id, ["general"]);
}

const showGameLabel = computed(() => !!props.game);
const showLoaderLabel = computed(() => !!props.loader);

let projectData: Ref<Project | null>;
if (props.upstream) {
  const { data } = await useAsyncData<Project>(
    `server-project-${props.server_id}`,
    async (): Promise<Project> => {
      const result = await useBaseFetch(`project/${props.upstream?.project_id}`);
      return result as Project;
    },
  );
  projectData = data;
} else {
  projectData = ref(null);
}

const image = useState<string | undefined>(`server-icon-${props.server_id}`, () => undefined);

if (import.meta.server && projectData.value?.icon_url) {
  await useModrinthServers(props.server_id!, ["general"]);
}

const iconUrl = computed(() => projectData.value?.icon_url || undefined);
const isConfiguring = computed(() => props.flows?.intro);
</script>
