<template>
  <div>
    <div v-if="data && status === 'success'">
      <div class="relative w-full overflow-hidden rounded-2xl bg-bg-raised p-8">
        <div class="flex flex-col gap-4">
          <div class="experimental-styles-within flex flex-row items-center justify-between">
            <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">
              Current instance
            </h2>
            <Button color="secondary">
              <EditIcon />
              Edit
            </Button>
          </div>
          <p>
            {{ formatMessage(messages.currentInstanceDescription) }}
          </p>
        </div>
        <ul
          class="display-mode--list [&>article[class*='project-card_base-card_padding-bg']]:mt-4 [&>article[class*='project-card_base-card_padding-bg']]:bg-button-bg [&>article[class*='project-card_base-card_padding-bg']]:shadow-none"
        >
          <UiProjectCard
            v-if="data && data.project"
            :id="data.project.id"
            :icon-url="data.project.icon_url"
            :name="data.project.title"
            :description="data.project.description"
            :client-side="data.project.client_side"
            :server-side="data.project.server_side"
            type="modpack"
            :show-updated-date="false"
            :show-created-date="false"
          />
        </ul>
      </div>
    </div>
    <PyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
import { Button } from "@modrinth/ui";
import { EditIcon } from "@modrinth/assets";
import { useServerStore } from "~/stores/servers";
import PyroLoading from "~/components/ui/servers/PyroLoading.vue";

const { formatMessage } = useVIntl();
const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

await serverStore.fetchServerData(serverId);
const { data, status } = await useLazyAsyncData("contentServerData", async () =>
  serverStore.getServerData(serverId),
);

const messages = defineMessage({
  currentInstanceLabel: {
    id: "servers.manage.content.currentInstanceLabel",
    defaultMessage: "Current Instance",
  },
  currentInstanceDescription: {
    id: "servers.manage.content.currentInstanceDescription",
    defaultMessage: "Manage your server's content, like mods and plugins, in the Modrinth App.",
  },
});
</script>
