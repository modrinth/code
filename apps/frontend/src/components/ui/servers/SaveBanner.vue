<template>
  <transition name="save-banner">
    <div
      v-if="props.isVisible"
      data-pyro-save-banner
      class="fixed bottom-8 left-4 right-4 z-50 mx-auto h-fit w-full max-w-4xl rounded-2xl border-2 border-solid border-button-border bg-bg-raised p-4 transition-all duration-300"
    >
      <div class="flex flex-col items-center justify-between gap-2 md:flex-row">
        <span class="font-bold text-contrast">Careful, you have unsaved changes!</span>
        <div class="flex gap-2">
          <ButtonStyled type="transparent" color="standard" transparent>
            <button :disabled="props.isUpdating" @click="props.reset">Reset</button>
          </ButtonStyled>
          <ButtonStyled type="standard" color="brand">
            <button :disabled="props.isUpdating" @click="props.save">
              {{ props.isUpdating ? "Saving..." : "Save" }}
            </button>
          </ButtonStyled>
          <ButtonStyled v-if="props.restart" type="standard" color="brand">
            <button :disabled="props.isUpdating" @click="saveAndRestart">
              {{ props.isUpdating ? "Saving..." : "Save & Restart" }}
            </button>
          </ButtonStyled>
        </div>
      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  isUpdating: boolean;
  restart?: boolean;
  save: () => void;
  reset: () => void;
  isVisible: boolean;
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
}>();

const saveAndRestart = async () => {
  props.save();
  await props.server.general?.power("Restart");
};
</script>

<style scoped>
.save-banner-enter-active {
  transition:
    opacity 400ms,
    transform 400ms;
}

.save-banner-leave-active {
  transition:
    opacity 200ms,
    transform 200ms;
}

.save-banner-enter-from,
.save-banner-leave-to {
  opacity: 0;
  transform: translateY(100%) scale(0.98);
}

.save-banner-enter-to,
.save-banner-leave-from {
  opacity: 1;
  transform: none;
}
</style>
