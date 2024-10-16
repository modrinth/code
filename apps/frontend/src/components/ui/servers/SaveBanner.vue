<template>
  <div
    data-pyro-save-banner
    class="save-banner fixed bottom-8 left-4 right-4 z-50 mx-auto h-fit w-full max-w-4xl rounded-2xl border-2 border-solid border-divider bg-bg-raised p-4 transition-all duration-300"
  >
    <div class="flex flex-col items-center justify-between gap-2 md:flex-row">
      <span class="font-bold text-contrast">Careful, you have unsaved changes!</span>
      <div class="flex gap-2">
        <ButtonStyled type="transparent" color="standard" transparent :disabled="props.isUpdating">
          <button @click="props.reset">Reset</button>
        </ButtonStyled>
        <ButtonStyled type="standard" color="brand" :disabled="props.isUpdating">
          <button @click="props.save">{{ props.isUpdating ? "Saving..." : "Save" }}</button>
        </ButtonStyled>
        <ButtonStyled
          v-if="props.restart"
          type="standard"
          color="brand"
          :disabled="props.isUpdating"
        >
          <button @click="saveAndRestart">
            {{ props.isUpdating ? "Saving..." : "Save & Restart" }}
          </button>
        </ButtonStyled>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  isUpdating: boolean;
  restart?: boolean;
  save: () => void;
  reset: () => void;
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
}>();

const saveAndRestart = async () => {
  props.save();
  await props.server.general?.power("Restart");
};
</script>

<style scoped>
.save-banner {
  animation: slide-up 200ms ease;
}

@keyframes slide-up {
  from {
    opacity: 0;
    transform: translateY(100%);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
