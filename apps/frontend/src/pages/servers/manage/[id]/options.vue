<template>
  <div class="flex gap-4">
    <div class="normal-page__sidebar">
      <aside class="universal-card">
        <NavStack>
          <NavStackItem :link="`/servers/manage/${serverId}/options`" label="General">
            <GeneralIcon />
          </NavStackItem>
          <NavStackItem :link="`/servers/manage/${serverId}/options/properties`" label="Properties">
            <PropertiesIcon />
          </NavStackItem>
          <NavStackItem :link="`/servers/manage/${serverId}/options/info`" label="Info">
            <InfoIcon />
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

import GeneralIcon from "~/assets/images/utils/navtabs/cog.svg?component";
import InfoIcon from "~/assets/images/utils/file.svg?component";
import PropertiesIcon from "~/assets/images/utils/list.svg?component";

const route = useNativeRoute();
const serverId = route.params.id;
</script>
