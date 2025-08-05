<template>
  <NuxtLink
    class="contents"
    :to="status === 'suspended' ? '' : `/servers/manage/${props.server_id}`"
  >
    <div
      class="medal-promotion flex flex-row items-center overflow-x-hidden rounded-2xl p-4 shadow-xl transition-transform duration-100"
      :class="status === 'suspended' ? '!rounded-b-none border-b-0 opacity-75' : 'active:scale-95'"
      data-pyro-server-listing
      :data-pyro-server-listing-id="server_id"
    >
      <div class="overlay"></div>
      <MedalPromoBackground class="background-pattern" />
      <MedalServerIcon
        v-if="status !== 'suspended'"
        class="z-10 size-16 rounded-xl bg-bg text-orange"
      />
      <div
        v-else
        class="bg-bg-secondary z-10 flex size-16 items-center justify-center rounded-xl border-[1px] border-solid border-button-border bg-button-bg shadow-sm"
      >
        <LockIcon class="size-12 text-secondary" />
      </div>
      <div class="z-10 ml-4 flex flex-col gap-2.5">
        <div class="flex flex-row items-center gap-2">
          <h2 class="m-0 text-xl font-bold text-contrast">{{ name }}</h2>
          <ChevronRightIcon />

          <span>{{ timeLeftCountdown }}</span>
        </div>

        <div
          v-if="projectData?.title"
          class="m-0 flex flex-row items-center gap-2 text-sm font-medium text-secondary"
        >
          <Avatar
            :src="iconUrl"
            no-shadow
            style="min-height: 20px; min-width: 20px; height: 20px; width: 20px"
            alt="Server Icon"
          />
          Using {{ projectData?.title || "Unknown" }}
        </div>

        <div
          v-if="isConfiguring"
          class="flex min-w-0 items-center gap-2 truncate text-sm font-semibold text-brand"
        >
          <SparklesIcon class="size-5 shrink-0" /> New server
        </div>
        <UiServersServerInfoLabels
          v-else
          :server-data="{ game, mc_version, loader, loader_version, net }"
          :show-game-label="showGameLabel"
          :show-loader-label="showLoaderLabel"
          :linked="false"
          class="pointer-events-none flex w-full flex-row flex-wrap items-center gap-4 text-secondary *:hidden sm:flex-row sm:*:flex"
        />
      </div>
    </div>
    <div
      v-if="status === 'suspended' && suspension_reason === 'upgrading'"
      class="relative -mt-4 flex w-full flex-row items-center gap-2 rounded-b-2xl border-[1px] border-t-0 border-solid border-bg-blue bg-bg-blue p-4 text-sm font-bold text-contrast"
    >
      <UiServersPanelSpinner />
      Your server's hardware is currently being upgraded and will be back online shortly.
    </div>
    <div
      v-else-if="status === 'suspended' && suspension_reason === 'cancelled'"
      class="relative -mt-4 flex w-full flex-col gap-2 rounded-b-2xl border-[1px] border-t-0 border-solid border-bg-red bg-bg-red p-4 text-sm font-bold text-contrast"
    >
      <div class="flex flex-row gap-2">
        <UiServersIconsPanelErrorIcon class="!size-5" /> Your server has been cancelled. Please
        update your billing information or contact Modrinth Support for more information.
      </div>
      <CopyCode :text="`${props.server_id}`" class="ml-auto" />
    </div>
    <div
      v-else-if="status === 'suspended' && suspension_reason"
      class="relative -mt-4 flex w-full flex-col gap-2 rounded-b-2xl border-[1px] border-t-0 border-solid border-bg-red bg-bg-red p-4 text-sm font-bold text-contrast"
    >
      <div class="flex flex-row gap-2">
        <UiServersIconsPanelErrorIcon class="!size-5" /> Your server has been suspended:
        {{ suspension_reason }}. Please update your billing information or contact Modrinth Support
        for more information.
      </div>
      <CopyCode :text="`${props.server_id}`" class="ml-auto" />
    </div>
    <div
      v-else-if="status === 'suspended'"
      class="relative -mt-4 flex w-full flex-col gap-2 rounded-b-2xl border-[1px] border-t-0 border-solid border-bg-red bg-bg-red p-4 text-sm font-bold text-contrast"
    >
      <div class="flex flex-row gap-2">
        <UiServersIconsPanelErrorIcon class="!size-5" /> Your server has been suspended. Please
        update your billing information or contact Modrinth Support for more information.
      </div>
      <CopyCode :text="`${props.server_id}`" class="ml-auto" />
    </div>
  </NuxtLink>
</template>

<script setup lang="ts">
import { ChevronRightIcon, LockIcon, SparklesIcon } from "@modrinth/assets";
import type { Project, Server } from "@modrinth/utils";
import { Avatar, CopyCode } from "@modrinth/ui";
import dayjs from "dayjs";
import MedalServerIcon from "~/assets/images/servers/medal_server_icon.svg?component";
import MedalPromoBackground from "~/assets/images/illustrations/medal_promo_background.svg?component";

const props = defineProps<Partial<Server>>();

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

const iconUrl = computed(() => projectData.value?.icon_url || undefined);
const isConfiguring = computed(() => props.flows?.intro);

const expiryDate = dayjs().add(5, "day");
const timeLeftCountdown = computed(() => {
  const now = dayjs();
  const diff = expiryDate.diff(now, "day");
  return diff > 0 ? `${diff} day${diff > 1 ? "s" : ""} left` : "Expired";
});
</script>

<style scoped lang="scss">
.medal-promotion {
  position: relative;
  border: 1px solid var(--medal-promotion-bg-orange);
  background: inherit; // allows overlay + pattern to take over
}
.overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: linear-gradient(
    90deg,
    var(--medal-promotion-bg) 10%,
    transparent 30%,
    var(--medal-promotion-bg) 100%
  );
  z-index: 1;
  border-radius: inherit;
}
.background-pattern {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 0;
  background-color: var(--medal-promotion-bg);
  border-radius: inherit;
  color: var(--medal-promotion-text-orange);
}
</style>
