<template>
  <div
    class="absolute z-50 flex h-full min-h-[400px] w-full flex-1 items-center justify-center"
    role="alertdialog"
    aria-labelledby="error-title"
    aria-describedby="error-message"
  >
    <div
      class="pyro-logo-animation relative flex w-[35rem] flex-col items-center gap-4 rounded-2xl border-2 border-solid border-[#FF496E] bg-[#fff5f6] p-8 dark:bg-[#270B11]"
      tabindex="-1"
    >
      <UiServersIconsPanelErrorIcon class="size-8" />
      <h1
        id="error-title"
        class="m-0 inline-flex items-center gap-2 text-4xl font-bold dark:text-[var(--color-contrast)]"
      >
        {{ title }}
      </h1>
      <h2
        id="error-message"
        class="m-0 max-w-md text-center text-base font-normal leading-relaxed text-secondary"
      >
        {{ message }}
      </h2>
      <p v-if="serverId" class="m-0 max-w-md text-center leading-relaxed text-secondary">
        If this issue persists, contact Modrinth support and provide the following server ID:
        <UiCopyCode :text="serverId" />
      </p>
      <p v-else class="m-0 max-w-md text-center leading-relaxed text-secondary">
        If this issue persists, contact Modrinth support.
      </p>
      <div class="flex flex-row gap-4">
        <ButtonStyled type="standard">
          <button aria-label="Back to servers" @click="$router.push('/servers/manage')">
            <LeftArrowIcon class="h-6 w-6" aria-hidden="true" />
            Back to servers
          </button>
        </ButtonStyled>
        <ButtonStyled type="standard" color="brand">
          <button aria-label="Reload" @click="reloadNuxtApp()">
            <UpdatedIcon class="h-6 w-6" aria-hidden="true" />
            Reload
          </button>
        </ButtonStyled>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { LeftArrowIcon, UpdatedIcon } from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";

defineProps({
  title: {
    type: String,
    required: true,
  },
  message: {
    type: String,
    required: true,
  },
  serverId: {
    type: String,
    default: "",
    required: false,
  },
});
</script>

<style scoped>
.page-enter-active,
.page-leave-active {
  transition: all 0.1s;
}

.page-enter-from,
.page-leave-to {
  opacity: 0;
}

@keyframes zoom-in {
  0% {
    transform: scale(0.5);
  }

  100% {
    transform: scale(1);
  }
}

.pyro-logo-animation {
  animation: zoom-in 0.8s
    linear(
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
    );
}

@keyframes fade-bg-in {
  0% {
    opacity: 0;
  }

  100% {
    opacity: 0.6;
  }
}

.bg-loading-animation {
  animation: fade-bg-in 0.12s linear forwards;
}
</style>
