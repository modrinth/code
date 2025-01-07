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
      class="flex cursor-pointer flex-row items-center overflow-x-hidden rounded-3xl bg-bg-raised p-4 transition-transform duration-100"
      :class="status === 'suspended' ? '!rounded-b-none opacity-75' : 'active:scale-95'"
      data-pyro-server-listing
      :data-pyro-server-listing-id="server_id"
    >
      <UiServersServerIcon v-if="status !== 'suspended'" :image="image" />
      <div
        v-else
        class="bg-bg-secondary flex size-24 items-center justify-center rounded-xl border-[1px] border-solid border-button-border bg-button-bg shadow-sm"
      >
        <LockIcon class="size-20 text-secondary" />
      </div>
      <div class="ml-8 flex flex-col gap-2.5">
        <div class="flex flex-row items-center gap-2">
          <h2 class="m-0 text-xl font-bold text-contrast">{{ name }}</h2>
          <ChevronRightIcon />
        </div>

        <div
          v-if="projectData?.title"
          class="m-0 flex flex-row items-center gap-2 text-sm font-medium text-secondary"
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

        <UiServersServerInfoLabels
          :server-data="{ game, mc_version, loader, loader_version, net }"
          :show-game-label="showGameLabel"
          :show-loader-label="showLoaderLabel"
          :show-subdomain-label="showSubdomainLabel"
          :linked="false"
          class="pointer-events-none flex w-full flex-row flex-wrap items-center gap-4 text-secondary *:hidden sm:flex-row sm:*:flex"
        />
      </div>
    </div>
    <div
      v-if="status === 'suspended' && suspension_reason === 'upgrading'"
      class="relative -mt-4 flex w-full flex-row items-center gap-2 rounded-b-3xl bg-bg-blue p-4 text-sm font-bold text-contrast"
    >
      <UiServersPanelSpinner />
      Your server's hardware is currently being upgraded and will be back online shortly.
    </div>
    <div
      v-if="status === 'suspended' && suspension_reason === 'support'"
      class="relative -mt-4 flex w-full flex-row items-center gap-2 rounded-b-3xl bg-bg-blue p-4 text-sm font-bold text-contrast"
    >
      <HammerIcon />
      You recently requested support for your server and we are actively working on it. It will be
      back online shortly.
    </div>
    <div
      v-else-if="status === 'suspended' && suspension_reason !== 'upgrading'"
      class="relative -mt-4 flex w-full flex-row items-center gap-2 rounded-b-3xl bg-bg-red p-4 text-sm font-bold text-contrast"
    >
      <UiServersIconsPanelErrorIcon class="!size-5" />
      Your server has been suspended due to a billing issue. Please visit your billing settings or
      contact Modrinth Support for more information.
    </div>
  </NuxtLink>
</template>

<script setup lang="ts">
import { ChevronRightIcon, HammerIcon, LockIcon } from "@modrinth/assets";
import type { Project, Server } from "~/types/servers";

const props = defineProps<Partial<Server>>();

const showGameLabel = computed(() => !!props.game);
const showLoaderLabel = computed(() => !!props.loader);
const showSubdomainLabel = computed(() => !!props.net?.domain);

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

const image = ref<string | undefined>();

onMounted(async () => {
  const auth = (await usePyroFetch(`servers/${props.server_id}/fs`)) as any;
  try {
    const fileData = await usePyroFetch(`/download?path=/server-icon-original.png`, {
      override: auth,
    });

    if (fileData instanceof Blob) {
      const canvas = document.createElement("canvas");
      const ctx = canvas.getContext("2d");
      const img = new Image();
      img.src = URL.createObjectURL(fileData);
      await new Promise<void>((resolve) => {
        img.onload = () => {
          canvas.width = 512;
          canvas.height = 512;
          ctx?.drawImage(img, 0, 0, 512, 512);
          const dataURL = canvas.toDataURL("image/png");
          image.value = dataURL;
          resolve();
        };
      });
    }
  } catch (error) {
    if (error instanceof PyroFetchError && error.statusCode === 404) {
      image.value = undefined;
    } else {
      console.error(error);
    }
  }
});

const iconUrl = computed(() => projectData.value?.icon_url || undefined);
</script>
