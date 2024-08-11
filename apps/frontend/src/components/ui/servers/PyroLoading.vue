<template>
  <div class="flex h-[70vh] w-full flex-col items-center justify-center">
    <PyroIcon class="pyro-logo-animation size-32 opacity-10" />
    <p
      class="text-sm transition"
      :class="{ 'opacity-0': !showLoading, 'animate-pulse opacity-100': showLoading }"
    >
      Loading...
    </p>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { PyroIcon } from "@modrinth/assets";

const showLoading = ref(false);

onMounted(() => {
  setTimeout(() => {
    showLoading.value = true;
  }, 5000);
});
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
