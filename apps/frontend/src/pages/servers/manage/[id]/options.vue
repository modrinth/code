<template>
  <div class="flex gap-4">
    <div class="normal-page__sidebar">
      <aside class="universal-card">
        <UiNavStack>
          <UiNavStackItem :link="`/servers/manage/${serverId}/options`" label="General">
            <CogIcon />
          </UiNavStackItem>
          <UiNavStackItem
            :link="`/servers/manage/${serverId}/options/args`"
            label="Launch Arguments"
          >
            <TextQuoteIcon />
          </UiNavStackItem>
          <UiNavStackItem :link="`/servers/manage/${serverId}/options/network`" label="Network">
            <VersionIcon />
          </UiNavStackItem>
          <UiNavStackItem
            :link="`/servers/manage/${serverId}/options/properties`"
            label="Properties"
          >
            <ListIcon />
          </UiNavStackItem>
          <UiNavStackItem :link="`/servers/manage/${serverId}/options/info`" label="Info">
            <FileIcon />
          </UiNavStackItem>
        </UiNavStack>
      </aside>
    </div>
    <div class="w-full">
      <NuxtPage :route="route" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { CogIcon, FileIcon, ListIcon, TextQuoteIcon, VersionIcon } from "@modrinth/assets";

const route = useNativeRoute();
const serverId = route.params.id;
</script>
