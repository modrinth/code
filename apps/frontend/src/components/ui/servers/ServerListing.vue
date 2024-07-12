<template>
  <NuxtLink
    :href="`/servers/manage/${serverId}`"
    class="flex flex-row items-center rounded-3xl bg-[var(--color-raised-bg)] p-4"
    data-pyro-server-listing
  >
    <UiAvatar no-shadow size="md" :src="''" alt="Server Icon" />
    <div class="ml-8 flex flex-col gap-3">
      <div class="flex flex-row items-center gap-2">
        <h2 class="m-0 text-xl font-bold">test</h2>
        <UiIconCaretRight />
      </div>

      <div class="flex flex-row items-center gap-4 text-[var(--color-text-secondary)]">
        <div class="flex flex-row items-center gap-2">
          <img
            src="~assets/images/games/minecraft-java-edition.png"
            alt="Minecraft Java Edition"
            class="size-5"
          />
          <span class="text-sm font-semibold">Minecraft</span>
        </div>

        <div class="h-6 w-0.5 bg-[var(--color-button-border)]"></div>
        <div class="flex flex-row items-center gap-2">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            xml:space="preserve"
            fill-rule="evenodd"
            stroke-linecap="round"
            stroke-linejoin="round"
            clip-rule="evenodd"
            viewBox="0 0 24 24"
            class="size-5"
          >
            <path fill="none" d="M0 0h24v24H0z"></path>
            <path
              fill="none"
              stroke="currentColor"
              stroke-width="23"
              d="m820 761-85.6-87.6c-4.6-4.7-10.4-9.6-25.9 1-19.9 13.6-8.4 21.9-5.2 25.4 8.2 9 84.1 89 97.2 104 2.5 2.8-20.3-22.5-6.5-39.7 5.4-7 18-12 26-3 6.5 7.3 10.7 18-3.4 29.7-24.7 20.4-102 82.4-127 103-12.5 10.3-28.5 2.3-35.8-6-7.5-8.9-30.6-34.6-51.3-58.2-5.5-6.3-4.1-19.6 2.3-25 35-30.3 91.9-73.8 111.9-90.8"
              transform="matrix(.08671 0 0 .0867 -49.8 -56)"
            ></path>
          </svg>
          <span class="text-sm font-semibold capitalize"> {{ loader }} {{ version }} </span>
        </div>

        <div class="h-6 w-0.5 bg-[var(--color-button-border)]"></div>
        <div class="flex flex-row items-center gap-2">
          <UiIconContentCube />
          <span class="text-sm font-semibold">
            {{ mods?.length }}
            <span v-if="mods?.length === 1">Mod</span>
            <span v-else>Mods</span>
          </span>
        </div>
      </div>
    </div>
  </NuxtLink>
</template>

<script setup lang="ts">
import type { Server as ServerType } from "~/types/servers";

type ServerProps = Omit<ServerType, "server_id"> & { serverId: string };

defineProps<ServerProps>();
</script>
