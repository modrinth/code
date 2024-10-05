<template>
  <div data-pyro class="servers-hero relative -mt-48 h-full min-h-screen py-48 md:-mt-20 md:py-32">
    <img
      src="~/assets/images/games/maze.png"
      alt=""
      aria-hidden="true"
      class="pointer-events-none absolute inset-0 h-full max-h-[1080px] w-screen scale-125 select-none opacity-50"
      style="mask-image: linear-gradient(black, transparent 80%)"
    />

    <section class="relative mx-auto flex max-w-7xl flex-col px-3">
      <div class="flex w-full flex-col gap-10">
        <div
          class="relative w-fit rounded-full bg-highlight-green px-3 py-1 text-sm font-bold text-brand backdrop-blur-lg"
        >
          Experimental Release
        </div>
        <h1 class="relative m-0 max-w-2xl text-4xl leading-[120%] md:text-7xl">
          Play together on Modrinth Servers
        </h1>
        <h2
          class="relative m-0 max-w-2xl text-base font-normal leading-[155%] text-secondary md:text-[1.2rem]"
        >
          Start your own Minecraft server directly in the Modrinth app. Play your favorite mods,
          plugins, and datapacks — without the hassle of setup.
        </h2>
        <div class="relative flex w-fit flex-row gap-8">
          <!--
            Open modal, choose plan
            Choose modpack (redirect to search page) - or choose a loader (forge/fabric/vanilla) with no mods
            Mount stripe
            On success, create server and redirect user to management page
          -->
          <ButtonStyled color="brand" size="large">
            <nuxt-link class="w-fit" to="/servers/new"> Start your server </nuxt-link>
          </ButtonStyled>
          <UiServersPoweredByPyro class="!mt-0" />
        </div>

        <div
          class="pointer-events-none relative flex h-full w-full flex-row items-end gap-24 md:-mt-24"
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
            class="relative mt-8 h-full w-full rounded-3xl object-cover md:w-[55%]"
          />
          <div class="relative hidden w-[45%] md:block">
            <img
              src="~/assets/images/games/combined-hero.png"
              alt=""
              class="relative w-full rounded-3xl object-fill"
            />
          </div>
        </div>
      </div>
    </section>

    <section
      class="relative mt-24 flex flex-col bg-[radial-gradient(65%_50%_at_50%_-10%,var(--color-brand-highlight)_0%,var(--color-accent-contrast)_100%)] px-3 pt-24 md:mt-48 md:pt-48"
    >
      <div class="faded-brand-line absolute top-0 h-[1px] w-full"></div>
      <div class="relative mx-auto flex w-full max-w-7xl flex-col gap-8">
        <div
          class="relative w-fit rounded-full bg-highlight-green px-3 py-1 text-sm font-bold text-brand backdrop-blur-lg"
        >
          Why Modrinth Servers?
        </div>
        <h1 class="relative m-0 max-w-2xl text-4xl leading-[120%] md:text-7xl">
          Make a modpack. Now it's a server.
        </h1>
        <h2
          class="relative m-0 max-w-2xl text-base font-normal leading-[155%] text-secondary md:text-[18px]"
        >
          Integrated with Modrinth App. Choose from thousands of modpacks or create your own from
          your profiles. Invite your friends when you're ready to play.
        </h2>
        <img
          src="~/assets/images/games/excitement.png"
          alt=""
          class="absolute right-8 top-0 hidden max-w-[360px] lg:block"
        />
        <div class="relative grid w-full grid-cols-1 gap-8 lg:grid-cols-2">
          <div class="relative flex flex-col gap-4 rounded-2xl bg-bg p-6 text-left md:p-12">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              class="size-8 text-brand"
            >
              <path
                d="M8.3 10a.7.7 0 0 1-.626-1.079L11.4 3a.7.7 0 0 1 1.198-.043L16.3 8.9a.7.7 0 0 1-.572 1.1Z"
              />
              <rect x="3" y="14" width="7" height="7" rx="1" />
              <circle cx="17.5" cy="17.5" r="3.5" />
            </svg>
            <h2 class="m-0 text-lg font-bold">Play where your mods are</h2>
            <h3 class="m-0 text-base font-normal text-secondary">
              Experience a seamless integration between your mods and your servers. Modpacks install
              within seconds and run without a hitch.
            </h3>
          </div>

          <div class="relative flex flex-col gap-4 rounded-2xl bg-bg p-6 text-left md:p-12">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              class="size-8 text-brand"
            >
              <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
              <circle cx="9" cy="7" r="4" />
              <path d="M22 21v-2a4 4 0 0 0-3-3.87" />
              <path d="M16 3.13a4 4 0 0 1 0 7.75" />
            </svg>
            <h2 class="m-0 text-lg font-bold">Join with a single click</h2>
            <h3 class="m-0 text-base font-normal text-secondary">
              Your servers are sharable with just one link. Easily invite your friends to join your
              server. Modrinth takes care of installing the mods.
            </h3>
          </div>
        </div>
        <div class="relative">
          <img
            src="~/assets/images/games/servers-hero-square-fixed-forreal.png"
            alt=""
            class="w-full rounded-2xl"
          />
        </div>
        <div class="grid w-full grid-cols-1 gap-8 lg:grid-cols-2">
          <div class="flex flex-col gap-4 rounded-2xl bg-bg p-6 text-left md:p-12">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              class="size-8 text-brand"
            >
              <rect width="20" height="16" x="2" y="4" rx="2" />
              <path d="M6 8h.01" />
              <path d="M10 8h.01" />
              <path d="M14 8h.01" />
            </svg>
            <h2 class="m-0 text-lg font-bold">Manage it all on Modrinth</h2>
            <h3 class="m-0 text-base font-normal text-secondary">
              Manage your server, mods, and players all in one place. No need to switch between
              different platforms just to play.
            </h3>
          </div>

          <div class="relative flex flex-col gap-4 rounded-2xl bg-bg p-6 text-left md:p-12">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              class="size-8 text-brand"
            >
              <polygon points="13 19 22 12 13 5 13 19" />
              <polygon points="2 19 11 12 2 5 2 19" />
            </svg>
            <h2 class="m-0 text-lg font-bold">Experience the fastest servers in the world</h2>
            <h3 class="m-0 text-base font-normal text-secondary">
              Modrinth Servers are hosted on the fastest, most reliable infrastructure using
              custom-built software built by Pyro.
            </h3>
          </div>
        </div>
      </div>
    </section>

    <section
      class="relative mt-24 flex flex-col bg-[radial-gradient(65%_50%_at_50%_-10%,var(--color-brand-highlight)_0%,var(--color-accent-contrast)_100%)] px-3 pt-24 md:mt-48 md:pt-48"
    >
      <div class="faded-brand-line absolute top-0 h-[1px] w-full"></div>
      <div class="relative mx-auto flex w-full max-w-7xl flex-col gap-8">
        <div
          class="relative w-fit rounded-full bg-highlight-green px-3 py-1 text-sm font-bold text-brand backdrop-blur-lg"
        >
          Included with your server
        </div>
        <h1 class="relative m-0 max-w-2xl text-4xl leading-[120%] md:text-7xl">
          All the features you need
        </h1>
        <h2
          class="relative m-0 max-w-xl text-base font-normal leading-[155%] text-secondary md:text-[18px]"
        >
          Included with every server is a suite of features designed to provide a hosting experience
          that only Modrinth can offer.
        </h2>
        <img
          src="~/assets/images/games/waving.png"
          alt=""
          class="absolute right-8 top-32 hidden max-w-[580px] lg:block"
        />
        <div class="grid grid-cols-1 gap-9 lg:grid-cols-2">
          <div class="grid w-full grid-cols-1 gap-8">
            <div class="relative flex flex-col gap-4 rounded-2xl bg-bg p-6 text-left md:p-12">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="size-8 text-brand"
              >
                <circle cx="12" cy="12" r="10" />
                <path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20" />
                <path d="M2 12h20" />
              </svg>
              <h2 class="m-0 text-lg font-bold">Custom subdomain</h2>
              <h3 class="m-0 text-base font-normal text-secondary">
                Share your server with a custom
                <span class="text-contrast">modrinth.gg</span> subdomain.
              </h3>
              <div
                aria-hidden="true"
                class="ooh-shiny absolute right-4 top-4 flex items-center justify-center rounded-full bg-bg-raised p-4"
              >
                <span class="font-bold text-contrast">{{ currentText }}</span
                >.modrinth.gg
              </div>
            </div>
            <div class="relative flex flex-col gap-4 rounded-2xl bg-bg p-6 text-left md:p-12">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="size-8 text-brand"
              >
                <path d="M12 13v8" />
                <path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242" />
                <path d="m8 17 4-4 4 4" />
              </svg>
              <h2 class="m-0 text-lg font-bold">Backups included</h2>
              <h3 class="m-0 text-base font-normal text-secondary">
                Backup your server up to 15 times and keep your data stored forever.
              </h3>
            </div>
          </div>
          <div
            class="relative flex flex-col gap-4 overflow-hidden rounded-2xl bg-highlight p-6 text-left backdrop-blur-xl md:p-12"
          >
            <h2 class="m-0 text-lg font-bold">Unlimited storage</h2>
            <h3 class="m-0 text-base font-normal">
              Store as many mods, plugins, and worlds as you need.
            </h3>

            <img
              src="~/assets/images/games/content-hero.png"
              alt=""
              class="absolute -bottom-16 -right-[15%] max-w-2xl rounded-2xl bg-brand p-4"
            />
            <div
              class="absolute bottom-8 right-8 rounded-full bg-brand p-4 text-sm font-bold text-[var(--color-accent-contrast)]"
            >
              8.49 GB used
            </div>
          </div>
        </div>
        <div class="relative flex flex-col gap-4 rounded-2xl bg-bg p-6 text-left md:p-12">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="size-8 text-brand"
          >
            <path
              d="M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z"
            />
            <path
              d="M12 5 9.04 7.96a2.17 2.17 0 0 0 0 3.08c.82.82 2.13.85 3 .07l2.07-1.9a2.82 2.82 0 0 1 3.79 0l2.96 2.66"
            />
            <path d="m18 15-2-2" />
            <path d="m15 18-2-2" />
          </svg>
          <h2 class="m-0 text-lg font-bold">Help when you need it</h2>
          <h3 class="m-0 text-base font-normal text-secondary">
            Reach out to the Modrinth team for help with your server.
          </h3>
        </div>
      </div>
    </section>

    <section
      class="relative mt-24 flex flex-col bg-[radial-gradient(65%_50%_at_50%_-10%,var(--color-brand-highlight)_0%,var(--color-accent-contrast)_100%)] px-3 pt-24 md:mt-48 md:pt-48"
    >
      <div class="faded-brand-line absolute top-0 h-[1px] w-full"></div>
      <div class="mx-auto flex w-full max-w-7xl flex-col items-center gap-8 text-center">
        <h1 class="relative m-0 text-4xl leading-[120%] md:text-7xl">
          Start your server on Modrinth
        </h1>
        <h2
          class="relative m-0 max-w-2xl text-base font-normal leading-[155%] text-secondary md:text-[18px]"
        >
          Plans start at ___. Cancel anytime.
        </h2>

        <div class="relative flex w-full flex-col items-center gap-8">
          <ButtonStyled color="brand" size="large">
            <nuxt-link class="w-fit" to="/mods"> Start your server </nuxt-link>
          </ButtonStyled>
          <UiServersPoweredByPyro class="!mt-0" />
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { ButtonStyled } from "@modrinth/ui";

const title = "Modrinth Servers";
const description =
  "Start your own Minecraft server directly in the Modrinth app. Play your favorite mods, plugins, and datapacks — without the hassle of setup.";

useSeoMeta({
  title,
  description,
  ogTitle: title,
  ogDescription: description,
});

const words = ["friends", "medieval-masters", "create-server", "mega-smp", "spookypack"];
const currentWordIndex = ref(0);
const currentText = ref("");
const isDeleting = ref(false);
const typingSpeed = 75;
const deletingSpeed = 25;
const pauseTime = 2000;

const startTyping = () => {
  const currentWord = words[currentWordIndex.value];
  if (isDeleting.value) {
    if (currentText.value.length > 0) {
      currentText.value = currentText.value.slice(0, -1);
      setTimeout(startTyping, deletingSpeed);
    } else {
      isDeleting.value = false;
      currentWordIndex.value = (currentWordIndex.value + 1) % words.length;
      setTimeout(startTyping, typingSpeed);
    }
  } else if (currentText.value.length < currentWord.length) {
    currentText.value = currentWord.slice(0, currentText.value.length + 1);
    setTimeout(startTyping, typingSpeed);
  } else {
    isDeleting.value = true;
    setTimeout(startTyping, pauseTime);
  }
};

onMounted(() => {
  document.body.style.background = "var(--color-accent-contrast)";
  const layoutDiv = document.querySelector(".layout") as HTMLElement;
  if (layoutDiv) {
    layoutDiv.style.background = "var(--color-accent-contrast)";
  }
  startTyping();
});

onUnmounted(() => {
  document.body.style.background = "";
  const layoutDiv = document.querySelector(".layout") as HTMLElement;
  if (layoutDiv) {
    layoutDiv.style.background = "";
  }
});
</script>

<style scoped>
.servers-hero {
  background: radial-gradient(
    65% 30% at 50% -10%,
    var(--color-brand-highlight) 0%,
    var(--color-accent-contrast) 100%
  );
}

.faded-brand-line {
  background: linear-gradient(to right, var(--color-brand-highlight), transparent);
}
</style>
