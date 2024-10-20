<template>
  <transition name="save-banner">
    <div
      v-if="props.isVisible"
      data-pyro-save-banner
      class="fixed bottom-8 left-4 right-4 z-50 mx-auto h-fit w-full max-w-4xl rounded-2xl border-2 border-solid border-divider bg-bg-raised p-4 transition-all duration-300"
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
    opacity 800ms,
    transform 800ms;
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

.save-banner-enter-to {
  transition-timing-function: linear(
    0 0%,
    0.01 0.8%,
    0.04 1.6%,
    0.161 3.3%,
    0.816 9.4%,
    1.046 11.9%,
    1.189 14.4%,
    1.231 15.7%,
    1.254 17%,
    1.259 17.8%,
    1.257 18.6%,
    1.236 20.45%,
    1.194 22.3%,
    1.057 27%,
    0.999 29.4%,
    0.955 32.1%,
    0.942 33.5%,
    0.935 34.9%,
    0.933 36.65%,
    0.939 38.4%,
    1 47.3%,
    1.011 49.95%,
    1.017 52.6%,
    1.016 56.4%,
    1 65.2%,
    0.996 70.2%,
    1.001 87.2%,
    1 100%
  ) !important;
}
</style>
