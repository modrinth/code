<template>
  <div class="relative w-full overflow-hidden rounded-2xl bg-bg-raised p-8">
    <div class="flex flex-col gap-4">
      <div class="experimental-styles-within flex flex-row items-center justify-between">
        <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">Current Instance</h2>
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
        :id="project.id"
        :icon-url="project.icon_url"
        :name="project.title"
        :description="project.description"
        :client-side="project.client"
        :server-side="project.server"
        type="modpack"
        :show-updated-date="false"
        :show-created-date="false"
      />
    </ul>
  </div>
</template>

<script setup lang="ts">
import { Button } from "@modrinth/ui";
import { EditIcon } from "@modrinth/assets";

const { formatMessage } = useVIntl();
const route = useNativeRoute();
const serverId = route.params.id;

const auth = await useAuth();

import type { Server } from "~/types/servers";

const data = await usePyroFetch<Server>(auth.value.token, `servers/${serverId}`);

const pid: any = await toRaw(useBaseFetch(`version/${data.modpack}`));
const project: any = await toRaw(useBaseFetch(`project/${pid.project_id}`));

console.log(project);

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
