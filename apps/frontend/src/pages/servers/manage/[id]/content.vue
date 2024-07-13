<template>
  <div class="relative w-full overflow-hidden rounded-2xl bg-[var(--color-raised-bg)] p-8">
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
      class="display-mode--list [&>article[class*='project-card_base-card_padding-bg']]:mt-4 [&>article[class*='project-card_base-card_padding-bg']]:bg-[var(--color-button-bg)] [&>article[class*='project-card_base-card_padding-bg']]:shadow-none"
    >
      <UiProjectCard
        :id="'23niDfW7'"
        icon-url="https://cdn.modrinth.com/data/23niDfW7/20644fac7c3890555049874f2cfb1040d10a5126.jpeg"
        name="Better Adventures++"
        description="A modified version of Flowstate's Better Adventures+ modpack that adds a few mods I feel keep the spirit of the original, while adding extra content, features, and structures"
        client-side="required"
        server-side="required"
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
