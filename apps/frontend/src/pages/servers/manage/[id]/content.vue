<template>
  <div class="relative w-full overflow-hidden rounded-2xl bg-[var(--color-raised-bg)] p-8">
    <div class="flex flex-col gap-4">
      <div class="experimental-styles-within flex flex-row items-center justify-between">
        <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">Current Instance</h2>
        <Button color="secondary">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 20 20"
            fill="currentColor"
            class="size-6"
          >
            <path
              d="m5.433 13.917 1.262-3.155A4 4 0 0 1 7.58 9.42l6.92-6.918a2.121 2.121 0 0 1 3 3l-6.92 6.918c-.383.383-.84.685-1.343.886l-3.154 1.262a.5.5 0 0 1-.65-.65Z"
            />
            <path
              d="M3.5 5.75c0-.69.56-1.25 1.25-1.25H10A.75.75 0 0 0 10 3H4.75A2.75 2.75 0 0 0 2 5.75v9.5A2.75 2.75 0 0 0 4.75 18h9.5A2.75 2.75 0 0 0 17 15.25V10a.75.75 0 0 0-1.5 0v5.25c0 .69-.56 1.25-1.25 1.25h-9.5c-.69 0-1.25-.56-1.25-1.25v-9.5Z"
            />
          </svg>
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
