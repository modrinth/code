<template>
  <div
    class="save-banner z-50 h-fit w-full rounded-xl border-2 border-solid border-divider bg-bg-raised p-4 transition-all duration-300"
  >
    <div class="flex items-center justify-between gap-2">
      <span class="font-bold text-contrast">Careful, you have unsaved changes!</span>
      <div class="flex gap-2">
        <Button transparent :loading="props.isUpdating" @click="props.reset"> Reset </Button>
        <Button color="primary" :loading="props.isUpdating" @click="props.save"> Save </Button>
        <Button
          v-if="props.restart"
          color="primary"
          :loading="props.isUpdating"
          @click="saveAndRestart"
        >
          Save & Restart
        </Button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button } from "@modrinth/ui";

const props = defineProps<{
  isUpdating: boolean;
  restart?: boolean;
  save: () => void;
  reset: () => void;
}>();

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const saveAndRestart = async () => {
  props.save();
  await serverStore.sendPowerAction(serverId, "Restart");
};
</script>

<style scoped>
.save-banner {
  animation: slide-up 0.3s ease;
}

@keyframes slide-up {
  from {
    transform: translateY(100%);
  }
  to {
    transform: translateY(0);
  }
}
</style>
