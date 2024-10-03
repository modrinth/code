<template>
  <div
    class="relative -mt-12 flex h-screen min-h-[400px] w-full flex-1 items-center justify-center"
    v-if="error"
  >
    <div
      class="bg-loading-animation absolute inset-0 -mt-8"
      style="
        background: linear-gradient(0deg, rgba(22, 24, 28, 0.64), rgba(22, 24, 28, 0.64)),
          linear-gradient(180deg, rgba(131, 66, 66, 0.275) 0%, rgba(202, 14, 14, 0.9) 97.29%);
      "
    ></div>
    <div
      class="bg-loading-animation pointer-events-none absolute inset-0 mx-auto flex h-full w-full max-w-7xl select-none items-center justify-center"
    >
      <img
        src="~/assets/images/games/bg-mock.png"
        alt="Background"
        class="absolute inset-0 mt-12 h-full w-full object-fill"
      />
    </div>
    <div
      class="pyro-logo-animation relative flex flex-col items-center gap-4 rounded-2xl border-2 border-solid border-[#FF496E] bg-[#270B11] p-8"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        stroke-width="1.5"
        stroke="currentColor"
        class="size-8 text-[#FF496E]"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z"
        />
      </svg>
      <h1
        class="m-0 inline-flex items-center gap-2 text-4xl font-bold text-[var(--color-contrast)]"
      >
        {{ error.name }}
      </h1>
      <div class="max-w-md text-center leading-relaxed text-secondary">
        {{ error.message }}
      </div>
      <div class="max-w-md text-center leading-relaxed text-secondary">
        If this issue persists, contact Modrinth support.
      </div>
      <Button @click="returnToServers">
        <LeftArrowIcon />
        Back to Servers
      </Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button } from "@modrinth/ui";
import { LeftArrowIcon } from "@modrinth/assets";

const route = useNativeRoute();
const router = useRouter();
const serverId = route.params.id as string;
const serverStore = useServerStore();

const error = ref<Error | null>(null);

watch(
  () => serverStore.error,
  (newVal) => {
    if (newVal) {
      error.value = newVal;
    }
  },
);
const returnToServers = () => {
  serverStore.clearError();
  router.push("/servers/manage");
};
</script>

<style>
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
