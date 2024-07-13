<template>
  <div class="flex gap-4">
    <div class="normal-page__sidebar">
      <aside class="universal-card">
        <NavStack>
          <NavStackItem :link="`/servers/manage/${serverId}/options`" label="General">
            <CogIcon />
          </NavStackItem>
          <NavStackItem :link="`/servers/manage/${serverId}/options/properties`" label="Properties">
            <ListIcon />
          </NavStackItem>
          <NavStackItem :link="`/servers/manage/${serverId}/options/info`" label="Info">
            <FileIcon />
          </NavStackItem>
        </NavStack>
      </aside>
    </div>
    <div class="w-full">
      <NuxtPage :route="route" />
    </div>
  </div>
</template>

<script setup lang="ts">
import NavStack from "~/components/ui/NavStack.vue";
import NavStackItem from "~/components/ui/NavStackItem.vue";

import { CogIcon, FileIcon, ListIcon } from "@modrinth/assets";

const route = useNativeRoute();
const serverId = route.params.id;
</script>
