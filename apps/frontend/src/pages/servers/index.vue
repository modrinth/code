<template>
  <div data-pyro class="servers-hero relative -mt-24 h-full min-h-screen pt-24">
    <PurchaseModal
      v-if="showModal && selectedProduct && customer"
      :key="selectedProduct.id"
      ref="purchaseModal"
      :product="selectedProduct"
      :country="country"
      :publishable-key="config.public.stripePublishableKey"
      :send-billing-request="
        async (body) =>
          await useBaseFetch('billing/payment', { internal: true, method: 'POST', body })
      "
      :fetch-payment-data="fetchPaymentData"
      :on-error="handleError"
      :customer="customer"
      :payment-methods="paymentMethods"
      :return-url="`${config.public.siteUrl}/servers/manage`"
      :server-name="`${auth?.user?.username}'s server`"
      @hidden="handleModalHidden"
    />

    <section
      class="relative mx-auto -mt-4 flex min-h-[calc(100vh)] max-w-7xl flex-col justify-center px-3"
    >
      <div class="relative z-10 flex w-full flex-col items-center gap-10 text-center">
        <div class="flex flex-col items-center gap-4">
          <div
            class="relative h-fit w-fit rounded-full bg-highlight-green px-3 py-1 text-sm font-bold text-brand backdrop-blur-lg"
          >
            Beta Release
          </div>
          <h1 class="relative m-0 max-w-4xl text-3xl font-bold !leading-[110%] md:text-7xl">
            Modrinth Servers
          </h1>
        </div>
        <h2
          class="relative m-0 max-w-3xl text-base font-normal leading-[155%] text-secondary md:text-[1.2rem]"
        >
          Start your own Minecraft server directly on Modrinth. Play with your favorite mods,
          plugins, and datapacks — without the hassle of setup — then invite your friends to join.
        </h2>
        <div class="relative flex w-full flex-wrap items-center gap-8 align-middle sm:w-fit">
          <div
            class="flex w-full flex-col items-center gap-5 text-center align-middle sm:w-fit sm:flex-row"
          >
            <ButtonStyled color="brand" size="large">
              <nuxt-link class="w-fit" to="#plan">
                <GameIcon aria-hidden="true" />
                {{ hasServers ? "Start a new server" : "Start your server" }}
              </nuxt-link>
            </ButtonStyled>
            <ButtonStyled v-if="hasServers" type="outlined" size="large">
              <nuxt-link class="w-fit" to="/servers/manage">
                <BoxIcon aria-hidden="true" /> Manage your servers
              </nuxt-link>
            </ButtonStyled>
          </div>
        </div>
        <UiServersPoweredByPyro class="!mt-0" />
      </div>
      <div
        class="pointer-events-none absolute left-0 right-0 top-0 z-0 flex h-screen w-full flex-row items-end gap-24"
      >
        <div
          class="absolute left-0 right-0 top-8 max-h-[90%] overflow-hidden sm:mt-0"
          style="mask-image: linear-gradient(black, transparent 80%)"
        >
          <img
            src="~/assets/images/games/rinth.png"
            alt=""
            aria-hidden="true"
            class="pointer-events-none w-full animate-spin select-none p-4 opacity-50"
            style="
              animation-duration: 172s !important;
              animation-timing-function: linear;
              animation-iteration-count: infinite;
            "
          />
        </div>
      </div>
    </section>

    <section
      class="relative mt-24 flex flex-col bg-[radial-gradient(65%_50%_at_50%_-10%,var(--color-brand-highlight)_0%,var(--color-accent-contrast)_100%)] px-3 pt-24 md:mt-48 md:pt-48"
    >
      <div class="faded-brand-line absolute left-0 top-0 h-[1px] w-full"></div>
      <div class="relative mx-auto flex w-full max-w-7xl flex-col gap-8">
        <div
          class="relative w-fit rounded-full bg-highlight-green px-3 py-1 text-sm font-bold text-brand backdrop-blur-lg"
        >
          Why Modrinth Servers?
        </div>
        <h1 class="relative m-0 max-w-2xl text-4xl leading-[120%] md:text-7xl">
          Find a modpack. Now it's a server.
        </h1>
        <h2
          class="relative m-0 max-w-2xl text-base font-normal leading-[155%] text-secondary md:text-[18px]"
        >
          Choose from the thousands of modpacks on Modrinth or create your own. Invite your friends
          when you're ready to play.
        </h2>
        <img
          src="~/assets/images/games/excitement.png"
          alt=""
          class="absolute right-14 top-0 hidden max-w-[360px] lg:block"
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
            <LoaderIcon loader="fabric" class="size-8 text-brand" />
            <h2 class="m-0 text-lg font-bold">All your favorite mods</h2>
            <h3 class="m-0 text-base font-normal text-secondary">
              Choose between Vanilla, Fabric, Forge, Quilt and NeoForge, so you never have to
              sacrifice any mod.
            </h3>
          </div>
        </div>
        <div class="relative">
          <img
            src="~/assets/images/games/servers-hero-square-fixed-forreal-updated.png"
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
            <h2 class="m-0 text-lg font-bold">
              Experience modern, reliable hosting powered by Pyro
            </h2>
            <h3 class="m-0 text-base font-normal text-secondary">
              Modrinth Servers are hosted on super-fast servers, with custom-built sofware to ensure
              your server runs smoothly.
            </h3>
          </div>
        </div>
      </div>
    </section>

    <section
      class="relative mt-24 flex flex-col bg-[radial-gradient(65%_50%_at_50%_-10%,var(--color-brand-highlight)_0%,var(--color-accent-contrast)_100%)] px-3 pt-24 md:mt-48 md:pt-48"
    >
      <div class="faded-brand-line absolute left-0 top-0 h-[1px] w-full"></div>
      <div class="relative mx-auto flex w-full max-w-7xl flex-col gap-8">
        <div
          class="relative w-fit rounded-full bg-highlight-green px-3 py-1 text-sm font-bold text-brand backdrop-blur-lg"
        >
          Included with your server
        </div>
        <h1 class="relative m-0 max-w-2xl text-4xl leading-[120%] md:text-7xl">
          Comes with all the features you need.
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
          class="absolute right-8 top-40 hidden max-w-[480px] lg:block"
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
              <h2 class="m-0 text-lg font-bold">Custom URL</h2>
              <h3 class="m-0 text-base font-normal text-secondary">
                Share your server with a custom
                <span class="text-contrast">modrinth.gg</span> URL.
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
                Every server comes with 15 backups stored off-site with Backblaze.
              </h3>
            </div>
          </div>
          <div
            style="
              background: radial-gradient(
                86.12% 101.64% at 95.97% 94.07%,
                rgba(27, 217, 106, 0.23) 0%,
                rgba(14, 115, 56, 0.2) 100%
              );
              border: 1px solid rgba(27, 217, 106, 0.07);
              box-shadow: 0px 12px 38.1px rgba(27, 217, 106, 0.13);
            "
            class="relative flex flex-col gap-4 overflow-hidden rounded-2xl p-6 text-left sm:backdrop-blur-xl md:p-12"
          >
            <h2 class="m-0 text-lg font-bold">Easy to use file manager</h2>
            <h3 class="m-0 text-base font-normal">
              Search, manage, and upload files directly to your server with ease.
            </h3>

            <img
              src="~/assets/images/games/content-hero-fixed.png"
              alt=""
              class="absolute -bottom-12 -right-[15%] hidden max-w-2xl rounded-2xl bg-brand p-4 lg:block"
            />
            <div class="flex flex-row items-center gap-3">
              <div
                aria-hidden="true"
                class="hidden max-w-fit rounded-full bg-brand p-4 text-sm font-bold text-[var(--color-accent-contrast)] sm:absolute sm:bottom-8 sm:right-8 lg:block"
              >
                8.49 GB used
              </div>
              <div
                aria-hidden="true"
                class="flex w-fit items-center gap-2 rounded-full bg-button-bg p-3 sm:hidden"
              >
                <SortAscendingIcon class="h-6 w-6" />
                Sort
              </div>
              <div
                aria-hidden="true"
                class="flex w-fit items-center rounded-full bg-button-bg p-3 sm:hidden"
              >
                <SearchIcon class="h-6 w-6" />
              </div>
            </div>
          </div>
        </div>
        <div class="grid w-full grid-cols-1 gap-8 lg:grid-cols-2">
          <div class="relative flex flex-col gap-4 rounded-2xl bg-bg p-6 text-left md:p-12">
            <!-- <svg
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
            </svg> -->
            <TerminalSquareIcon class="size-8 text-brand" />
            <h2 class="m-0 text-lg font-bold">
              An easy console, server properties manager, and more
            </h2>
            <h3 class="m-0 text-base font-normal text-secondary">
              Super powerful features with super simple access.
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
              Reach out to the Modrinth team for help with your server at any time.
            </h3>
          </div>
        </div>
      </div>
    </section>

    <section
      id="plan"
      class="relative mt-24 flex flex-col bg-[radial-gradient(65%_50%_at_50%_-10%,var(--color-brand-highlight)_0%,var(--color-accent-contrast)_100%)] px-3 pt-24 md:mt-48 md:pt-48"
    >
      <div class="faded-brand-line absolute left-0 top-0 h-[1px] w-full"></div>
      <div class="mx-auto flex w-full max-w-7xl flex-col items-center gap-8 text-center">
        <h1 class="relative m-0 text-4xl leading-[120%] md:text-7xl">
          Start your server on Modrinth
        </h1>
        <h2
          class="relative m-0 max-w-xl text-base font-normal leading-[155%] text-secondary md:text-[18px]"
        >
          {{
            isAtCapacity
              ? "We are currently at capacity. Please try again later."
              : "There's a plan for everyone! Choose the one that fits your needs."
          }}
        </h2>

        <ul class="m-0 flex w-full flex-col gap-8 p-0 md:flex-row">
          <li class="flex w-full flex-col gap-4 rounded-2xl bg-bg p-8 text-left md:w-1/3">
            <div class="flex flex-row items-center justify-between">
              <h1 class="m-0">Small</h1>
              <div
                class="grid size-8 place-content-center rounded-full bg-highlight-blue text-xs font-bold text-blue"
              >
                S
              </div>
            </div>
            <p class="m-0">
              Perfect for vanilla multiplayer, small friend groups, SMPs, and light modding.
            </p>
            <div class="flex flex-row flex-wrap items-center gap-3 text-nowrap">
              <p class="m-0">4 GB RAM</p>
              <div class="size-1.5 rounded-full bg-secondary opacity-25"></div>
              <p class="m-0">32 GB Storage</p>
              <div class="size-1.5 rounded-full bg-secondary opacity-25"></div>
              <p class="m-0">2 vCPUs</p>
            </div>
            <h2 class="m-0 text-3xl text-contrast">
              $12<span class="text-sm font-normal text-secondary">/month</span>
            </h2>
            <ButtonStyled color="blue" size="large">
              <button
                v-if="!isSmallAtCapacity"
                class="!bg-highlight-blue !font-medium !text-blue"
                @click="selectProduct(pyroProducts[0])"
              >
                Get Started
                <RightArrowIcon class="!min-h-4 !min-w-4" />
              </button>
              <a
                v-else
                href="https://support.modrinth.com"
                target="_blank"
                class="!bg-highlight-blue !font-medium !text-blue"
              >
                Out of Stock
                <ExternalIcon class="!min-h-4 !min-w-4" />
              </a>
            </ButtonStyled>
          </li>

          <li
            style="
              background: radial-gradient(
                86.12% 101.64% at 95.97% 94.07%,
                rgba(27, 217, 106, 0.23) 0%,
                rgba(14, 115, 56, 0.2) 100%
              );
              border: 1px solid rgb(12 107 52);
              box-shadow: 0px 12px 38.1px rgba(27, 217, 106, 0.13);
            "
            class="flex w-full flex-col gap-4 rounded-2xl bg-bg p-8 text-left md:w-1/3"
          >
            <div class="flex flex-row items-center justify-between">
              <h1 class="m-0">Medium</h1>
              <div
                class="grid size-8 place-content-center rounded-full bg-highlight-green text-xs font-bold text-brand"
              >
                M
              </div>
            </div>
            <p class="m-0">Great for modded multiplayer and small communities.</p>
            <div class="flex flex-row flex-wrap items-center gap-3 text-nowrap">
              <p class="m-0">6 GB RAM</p>
              <div class="size-1.5 rounded-full bg-secondary opacity-25"></div>
              <p class="m-0">48 GB Storage</p>
              <div class="size-1.5 rounded-full bg-secondary opacity-25"></div>
              <p class="m-0">3 vCPUs</p>
            </div>
            <h2 class="m-0 text-3xl text-contrast">
              $18<span class="text-sm font-normal text-secondary">/month</span>
            </h2>
            <ButtonStyled color="brand" size="large">
              <button
                v-if="!isMediumAtCapacity"
                class="shadow-xl"
                @click="selectProduct(pyroProducts[1])"
              >
                Get Started
                <RightArrowIcon class="!min-h-4 !min-w-4" />
              </button>
              <a
                v-else
                href="https://support.modrinth.com"
                target="_blank"
                class="!bg-highlight-green !font-medium !text-green"
              >
                Out of Stock
                <ExternalIcon class="!min-h-4 !min-w-4" />
              </a>
            </ButtonStyled>
          </li>

          <li class="flex w-full flex-col gap-4 rounded-2xl bg-bg p-8 text-left md:w-1/3">
            <div class="flex flex-row items-center justify-between">
              <h1 class="m-0">Large</h1>
              <div
                class="grid size-8 place-content-center rounded-full bg-highlight-purple text-xs font-bold text-purple"
              >
                L
              </div>
            </div>
            <p class="m-0">Ideal for larger communities, modpacks, and heavy modding.</p>
            <div class="flex flex-row flex-wrap items-center gap-3 text-nowrap">
              <p class="m-0">8 GB RAM</p>
              <div class="size-1.5 rounded-full bg-secondary opacity-25"></div>
              <p class="m-0">64 GB Storage</p>
              <div class="size-1.5 rounded-full bg-secondary opacity-25"></div>
              <p class="m-0">4 vCPUs</p>
            </div>
            <h2 class="m-0 text-3xl text-contrast">
              $24<span class="text-sm font-normal text-secondary">/month</span>
            </h2>
            <ButtonStyled color="purple" size="large">
              <button
                v-if="!isLargeAtCapacity"
                class="!bg-highlight-purple !font-medium !text-purple"
                @click="selectProduct(pyroProducts[2])"
              >
                Get Started
                <RightArrowIcon class="!min-h-4 !min-w-4" />
              </button>
              <a
                v-else
                href="https://support.modrinth.com"
                target="_blank"
                class="!bg-highlight-purple !font-medium !text-purple"
              >
                Out of Stock
                <ExternalIcon class="!min-h-4 !min-w-4" />
              </a>
            </ButtonStyled>
          </li>
        </ul>

        <div
          class="flex w-full flex-col items-start justify-between gap-4 rounded-2xl bg-bg p-8 text-left md:flex-row md:gap-0"
        >
          <div class="flex flex-col gap-4">
            <h1 class="m-0">Build your own</h1>
            <h2 class="m-0 text-base font-normal">
              If you're a more technical server administrator, you can pick your own RAM and storage
              options.
            </h2>
          </div>

          <div class="flex w-full flex-col-reverse gap-2 md:w-auto md:flex-col md:items-center">
            <ButtonStyled color="standard" size="large">
              <NuxtLink to="/servers/custom" class="w-full md:w-fit">
                Build your own
                <RightArrowIcon class="!min-h-4 !min-w-4" />
              </NuxtLink>
            </ButtonStyled>
            <p class="m-0 text-sm">Starting at $3/GB RAM</p>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup>
import { ButtonStyled, PurchaseModal } from "@modrinth/ui";
import {
  BoxIcon,
  GameIcon,
  RightArrowIcon,
  SearchIcon,
  SortAscendingIcon,
  ExternalIcon,
  TerminalSquareIcon,
} from "@modrinth/assets";
import { products } from "~/generated/state.json";
import LoaderIcon from "~/components/ui/servers/icons/LoaderIcon.vue";

const pyroProducts = products.filter((p) => p.metadata.type === "pyro");
pyroProducts.sort((a, b) => a.metadata.ram - b.metadata.ram);

const title = "Modrinth Servers";
const description =
  "Start your own Minecraft server directly on Modrinth. Play your favorite mods, plugins, and datapacks — without the hassle of setup.";

useSeoMeta({
  title,
  description,
  ogTitle: title,
  ogDescription: description,
});

useHead({
  script: [
    {
      src: "https://js.stripe.com/v3/",
      defer: true,
      async: true,
    },
  ],
});

const auth = await useAuth();
const data = useNuxtApp();
const config = useRuntimeConfig();
const purchaseModal = ref(null);
const country = useUserCountry();
const customer = ref(null);
const paymentMethods = ref([]);
const selectedProduct = ref(null);
const showModal = ref(false);
const modalKey = ref(0);

const words = ["my-smp", "medieval-masters", "create-server", "mega-smp", "spookypack"];
const currentWordIndex = ref(0);
const currentText = ref("");
const isDeleting = ref(false);
const typingSpeed = 75;
const deletingSpeed = 25;
const pauseTime = 2000;

const { data: hasServers } = await useAsyncData("ServerListCountCheck", async () => {
  try {
    if (!auth.value.user) return false;
    const response = await usePyroFetch("servers");
    return response.servers && response.servers.length > 0;
  } catch {
    return false;
  }
});

const { data: capacityStatuses } = await useAsyncData("ServerCapacityAll", async () => {
  try {
    const capacityChecks = pyroProducts.map((product) =>
      usePyroFetch("capacity", {
        method: "POST",
        body: {
          cpu: product.metadata.cpu,
          memory_mb: product.metadata.ram,
          swap_mb: product.metadata.swap,
          storage_mb: product.metadata.storage,
        },
      }),
    );

    const results = await Promise.all(capacityChecks);
    return {
      small: results[0],
      medium: results[1],
      large: results[2],
    };
  } catch (error) {
    console.error("Error checking server capacities:", error);
    return {
      small: { available: 0 },
      medium: { available: 0 },
      large: { available: 0 },
    };
  }
});

const isSmallAtCapacity = computed(() => capacityStatuses.value?.small?.available === 0);
const isMediumAtCapacity = computed(() => capacityStatuses.value?.medium?.available === 0);
const isLargeAtCapacity = computed(() => capacityStatuses.value?.large?.available === 0);

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

const handleError = (err) => {
  addNotification({
    group: "main",
    title: "An error occurred",
    type: "error",
    text: err.message ?? (err.data ? err.data.description : err),
  });
};

const handleModalHidden = () => {
  showModal.value = false;
};

watch(selectedProduct, async (newProduct) => {
  if (newProduct) {
    showModal.value = false;
    await nextTick();
    showModal.value = true;
    modalKey.value++;
    await nextTick();
    if (purchaseModal.value && purchaseModal.value.show) {
      purchaseModal.value.show();
    }
  }
});

async function fetchPaymentData() {
  if (!auth.value.user) return;
  try {
    const [customerData, paymentMethodsData] = await Promise.all([
      useBaseFetch("billing/customer", { internal: true }),
      useBaseFetch("billing/payment_methods", { internal: true }),
    ]);
    customer.value = customerData;
    paymentMethods.value = paymentMethodsData;
  } catch (error) {
    console.error("Error fetching payment data:", error);
    addNotification({
      group: "main",
      title: "Error fetching payment data",
      type: "error",
      text: error.message || "An unexpected error occurred",
    });
  }
}

const route = useRoute();
const isAtCapacity = computed(
  () => isSmallAtCapacity.value && isMediumAtCapacity.value && isLargeAtCapacity.value,
);

const selectProduct = async (product) => {
  if (isAtCapacity.value) {
    addNotification({
      group: "main",
      title: "Server Capacity Full",
      type: "error",
      text: "We are currently at capacity. Please try again later.",
    });
    return;
  }

  if (!auth.value.user) {
    data.$router.push(`/auth/sign-in?redirect=${encodeURIComponent("/servers?showModal=true")}`);
    return;
  }
  selectedProduct.value = product;
  showModal.value = true;
  modalKey.value++;
  await nextTick();
  if (purchaseModal.value && purchaseModal.value.show) {
    purchaseModal.value.show();
  }
};

const openPurchaseModal = () => {
  if (isAtCapacity.value) {
    addNotification({
      group: "main",
      title: "Server Capacity Full",
      type: "error",
      text: "We are currently at capacity. Please try again later.",
    });
    return;
  }

  selectedProduct.value = pyroProducts[0];
  showModal.value = true;
  modalKey.value++;
  nextTick(() => {
    if (purchaseModal.value && purchaseModal.value.show) {
      purchaseModal.value.show();
    }
  });
};

onMounted(() => {
  startTyping();
  if (route.query.showModal) {
    openPurchaseModal();
  }
});

watch(customer, (newCustomer) => {
  if (newCustomer) {
    if (route.query.showModal) {
      openPurchaseModal();
    }
  }
});

onMounted(() => {
  document.body.style.background = "var(--color-accent-contrast)";
  const layoutDiv = document.querySelector(".layout");
  if (layoutDiv) {
    layoutDiv.style.background = "var(--color-accent-contrast)";
  }
  fetchPaymentData();
});

onUnmounted(() => {
  document.body.style.background = "";
  const layoutDiv = document.querySelector(".layout");
  if (layoutDiv) {
    layoutDiv.style.background = "";
  }
  if (window.Stripe) {
    window.Stripe = null;
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
