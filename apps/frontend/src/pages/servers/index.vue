<template>
  <div class="servers-hero relative -mt-48 h-full min-h-screen py-48 md:-mt-20 md:py-32">
    <img
      src="~/assets/images/games/maze.png"
      alt=""
      aria-hidden="true"
      class="pointer-events-none absolute inset-0 h-full max-h-[1080px] w-screen scale-125 select-none opacity-50"
      style="mask-image: linear-gradient(black, transparent 80%)"
    />

    <section class="relative mx-auto flex max-w-7xl flex-col px-3">
      <div class="flex w-full flex-col gap-8">
        <div
          class="relative w-fit rounded-full bg-highlight-green px-3 py-1 text-sm font-bold text-brand backdrop-blur-lg"
        >
          Experimental Release
        </div>
        <h1 class="relative m-0 max-w-2xl text-4xl leading-[120%] md:text-7xl">
          Play together on Modrinth Servers
        </h1>
        <h2
          class="relative m-0 max-w-2xl text-base font-normal leading-[155%] text-secondary md:text-[18px]"
        >
          Start your own Minecraft server directly in the Modrinth app. Play your favorite mods,
          plugins, and datapacks — without the hassle of setup.
        </h2>
        <div class="relative flex w-fit flex-row gap-8">
          <ButtonStyled color="brand" size="large">
            <nuxt-link class="w-fit" to="/mods"> Start your server </nuxt-link>
          </ButtonStyled>
          <UiServersPoweredByPyro class="!mt-0" />
        </div>

        <div
          class="pointer-events-none relative -mt-24 flex h-full w-full flex-row items-end gap-24"
        >
          <div
            class="absolute left-0 right-0 top-0 max-h-[80%] overflow-hidden"
            style="mask-image: linear-gradient(black, transparent 80%)"
          >
            <img
              src="~/assets/images/games/rinth.png"
              alt=""
              aria-hidden="true"
              class="pointer-events-none w-full animate-spin select-none pt-8 opacity-50"
              style="
                animation-duration: 128s !important;
                animation-timing-function: linear;
                animation-iteration-count: infinite;
              "
            />
          </div>
          <img
            src="~/assets/images/games/servers-hero-real-noshadow.png"
            alt=""
            class="relative mt-8 h-full w-[55%] rounded-3xl object-cover"
          />
          <div class="relative w-[45%]">
            <img
              src="~/assets/images/games/combined-hero.png"
              alt=""
              class="relative w-full rounded-3xl object-fill"
            />
          </div>
        </div>
      </div>
    </section>

    <section class="relative mx-auto mt-48 flex max-w-7xl flex-col px-3">
      <div class="flex w-full flex-col items-center justify-center gap-8 text-center">
        <h1 class="relative m-0 max-w-2xl text-4xl leading-[120%] md:text-7xl">
          Make a modpack. Now it's a server.
        </h1>
        <h2
          class="relative m-0 max-w-lg text-base font-normal leading-[155%] text-secondary md:text-[18px]"
        >
          Integrated with the Modrinth App. Choose from thousands of modpacks or create your own.
          Invite your friends when you’re ready to play. It’s that easy.
        </h2>
        <ButtonStyled color="brand" size="large">
          <nuxt-link class="w-fit" to="/mods"> Get Started </nuxt-link>
        </ButtonStyled>
        <img src="~/assets/images/games/app-servers-hero.png" alt="" class="w-full" />
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ButtonStyled } from "@modrinth/ui";
</script>

<style scoped>
.servers-hero {
  background: radial-gradient(
    65% 30% at 50% -10%,
    var(--color-brand-highlight) 0%,
    var(--color-accent-contrast) 100%
  );
}
</style>
