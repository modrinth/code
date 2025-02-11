<template>
  <div
    ref="scrollListener"
    data-pyro
    class="servers-hero relative isolate -mt-44 h-full min-h-screen pt-8"
  >
    <PurchaseModal
      v-if="showModal && selectedProduct && customer"
      :key="selectedProduct.id"
      ref="purchaseModal"
      :product="selectedProduct"
      :country="country"
      :custom-server="customServer"
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
      :fetch-capacity-statuses="fetchCapacityStatuses"
      :out-of-stock-url="outOfStockUrl"
      @hidden="handleModalHidden"
    />

    <section
      class="mx-auto mt-32 flex min-h-[calc(80vh-0px)] max-w-7xl flex-col justify-center px-5 sm:mt-20 sm:min-h-[calc(100vh-0px)] sm:pl-10 lg:pl-3"
    >
      <div class="z-[5] flex w-full flex-col gap-8">
        <div class="flex flex-col gap-4">
          <div
            class="relative h-fit w-fit rounded-full bg-highlight-green px-3 py-1 text-sm font-bold text-brand backdrop-blur-lg"
          >
            Beta Release
          </div>
          <h1 class="relative m-0 max-w-3xl text-3xl font-bold !leading-[110%] md:text-6xl">
            Host your next server with Modrinth Servers
          </h1>
        </div>
        <h2
          class="relative m-0 max-w-2xl text-base font-normal leading-[155%] text-secondary md:text-[1.2rem]"
        >
          Modrinth Servers is the easiest way to host your own Minecraft server. Seamlessly install
          and play your favorite mods and modpacks, all within the Modrinth platform.
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
            <UiServersPoweredByPyro class="mx-0 !mt-0" />
          </div>
        </div>
      </div>

      <div
        class="absolute left-[55%] top-56 z-[5] hidden h-full max-h-[calc(100vh-10rem)] w-full rotate-1 xl:block"
      >
        <img
          src="https://cdn.modrinth.com/servers/panel-right-dark.webp"
          alt=""
          aria-hidden="true"
          class="pointer-events-none h-full w-fit select-none"
        />
      </div>

      <div
        class="top-26 pointer-events-none absolute left-0 z-[4] flex h-screen w-full flex-row items-end gap-24 sm:-right-1/4 sm:top-14"
      >
        <div
          class="pointer-events-none absolute left-0 right-0 top-8 max-h-[90%] overflow-hidden sm:top-28 sm:mt-0"
          style="mask-image: linear-gradient(black, transparent 80%)"
        >
          <img
            src="https://cdn.modrinth.com/servers/bigrinth.webp"
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
      class="relative flex flex-col bg-[radial-gradient(65%_50%_at_50%_-10%,var(--color-brand-highlight)_0%,var(--color-accent-contrast)_100%)] px-3 pt-24 md:pt-48"
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
          src="https://cdn.modrinth.com/servers/excitement.webp"
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
              Modrinth Servers seamlessly integrates the mod and modpack installation process into
              your server.
            </h3>
          </div>

          <div class="relative flex flex-col gap-4 rounded-2xl bg-bg p-6 text-left md:p-12">
            <LoaderIcon loader="fabric" class="size-8 text-brand" />
            <h2 class="m-0 text-lg font-bold">All your favorite mods</h2>
            <h3 class="m-0 text-base font-normal text-secondary">
              Choose between Vanilla, Fabric, Forge, Quilt and NeoForge. If it's on Modrinth, it can
              run on your server.
            </h3>
          </div>
        </div>
        <div class="relative">
          <img
            src="https://cdn.modrinth.com/servers/installation-dark.webp"
            alt=""
            class="hidden w-full rounded-2xl sm:block"
          />
        </div>
        <div class="grid w-full grid-cols-1 gap-8 lg:grid-cols-3">
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
              Your server, mods, players, and more are all on Modrinth. No need to switch between
              platforms.
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
              Modrinth Servers are hosted on
              <span class="text-contrast">2023 Ryzen 7/9 CPUs with DDR5 RAM</span>, running on
              custom-built software to ensure your server performs smoothly.
            </h3>
          </div>

          <div class="relative flex flex-col gap-4 rounded-2xl bg-bg p-6 text-left md:p-12">
            <ServerIcon class="size-8 text-brand" />
            <h2 class="m-0 text-lg font-bold">Consistently fast</h2>
            <h3 class="m-0 text-base font-normal text-secondary">
              Under Pyro, infrastructure is never overloaded, meaning each Modrinth server always
              runs at its full performance.
              <a
                class="mt-2 flex items-center gap-2 font-medium text-contrast transition-all hover:gap-3"
                href="https://status.pyro.host/"
                target="_blank"
              >
                See the infrastructure <RightArrowIcon class="flex-none" />
              </a>
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
          src="https://cdn.modrinth.com/servers/waving.webp"
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
                Every server comes with 15 backups stored securely off-site with Backblaze.
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
              border: 1px solid rgba(12, 107, 52, 0.55);
              box-shadow: 0px 12px 38.1px rgba(27, 217, 106, 0.13);
            "
            class="relative flex flex-col gap-4 overflow-hidden rounded-2xl p-6 text-left sm:backdrop-blur-xl md:p-12"
          >
            <h2 class="m-0 text-lg font-bold">Easy to use file manager</h2>
            <h3 class="m-0 text-base font-normal">
              Search, manage, edit, and upload files directly to your server with ease.
            </h3>

            <img
              src="https://cdn.modrinth.com/servers/content-dark.webp"
              alt=""
              class="absolute -bottom-12 -right-[15%] hidden max-w-2xl rounded-2xl bg-brand p-4 lg:block"
            />
            <div class="flex flex-row items-center gap-3">
              <div
                aria-hidden="true"
                class="max-w-fit rounded-full bg-brand p-4 text-sm font-bold text-[var(--color-accent-contrast)] lg:absolute lg:bottom-8 lg:right-8 lg:block"
              >
                8.49 GB used
              </div>
              <div
                aria-hidden="true"
                class="flex w-fit items-center gap-2 rounded-full bg-button-bg p-3 lg:hidden"
              >
                <SortAscendingIcon class="h-6 w-6" />
                Sort
              </div>
              <div
                aria-hidden="true"
                class="flex w-fit items-center rounded-full bg-button-bg p-3 lg:hidden"
              >
                <SearchIcon class="h-6 w-6" />
              </div>
            </div>
          </div>
        </div>
        <div class="grid w-full grid-cols-1 gap-8 lg:grid-cols-2">
          <div class="relative flex flex-col gap-4 rounded-2xl bg-bg p-6 text-left md:p-12">
            <TerminalSquareIcon class="size-8 text-brand" />
            <h2 class="m-0 text-lg font-bold">
              A powerful console, server properties manager, and more
            </h2>
            <h3 class="m-0 text-base font-normal text-secondary">
              Modrinth Servers come with powerful tools to manage your server.
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

        <div class="grid w-full grid-cols-1 gap-8 lg:grid-cols-2">
          <div class="relative flex flex-col gap-4 rounded-2xl bg-bg p-6 text-left md:p-12">
            <TransferIcon class="size-8 text-brand" />
            <h2 class="m-0 text-lg font-bold">SFTP access</h2>
            <h3 class="m-0 text-base font-normal text-secondary">
              Access your server files directly with SFTP built into Modrinth Servers.
            </h3>
          </div>
          <div class="relative flex flex-col gap-4 rounded-2xl bg-bg p-6 text-left md:p-12">
            <VersionIcon class="size-8 text-brand" />
            <h2 class="m-0 text-lg font-bold">Advanced networking management</h2>
            <h3 class="m-0 text-base font-normal text-secondary">
              Add your own domain to your server, reserve up to 15 ports for mods that require them,
              and more.
            </h3>
          </div>
        </div>
        <div class="relative flex flex-col gap-4 rounded-2xl bg-bg p-6 text-left md:p-12">
          <h1 class="m-0 text-lg font-bold">Frequently Asked Questions</h1>
          <div class="details-hide flex flex-col gap-1">
            <details pyro-hash="cpus" class="group" :open="$route.hash === '#cpus'">
              <summary class="flex cursor-pointer items-center py-3 font-medium text-contrast">
                <span class="mr-2 transition-transform duration-200 group-open:rotate-90">
                  <RightArrowIcon />
                </span>
                What kind of CPUs do Modrinth Servers run on?
              </summary>
              <p class="m-0 !leading-[190%]">
                Modrinth Servers use 2023 Ryzen 7 and Ryzen 9 CPUs at 4+ GHz, paired with DDR5
                memory.
              </p>
            </details>

            <details pyro-hash="ddos" class="group" :open="$route.hash === '#ddos'">
              <summary class="flex cursor-pointer items-center py-3 font-medium text-contrast">
                <span class="mr-2 transition-transform duration-200 group-open:rotate-90">
                  <RightArrowIcon />
                </span>
                Do Modrinth Servers have DDoS protection?
              </summary>
              <p class="m-0 !leading-[190%]">
                Yes. All Modrinth Servers come with DDoS protection. Protection is powered by a
                combination of in-house network filtering by Pyro as well as with our data center
                provider. Your server is safe on Modrinth.
              </p>
            </details>

            <details pyro-hash="region" class="group" :open="$route.hash === '#region'">
              <summary class="flex cursor-pointer items-center py-3 font-medium text-contrast">
                <span class="mr-2 transition-transform duration-200 group-open:rotate-90">
                  <RightArrowIcon />
                </span>
                Where are Modrinth Servers located? Can I choose a region?
              </summary>
              <p class="m-0 !leading-[190%]">
                Currently, Modrinth Servers are located in New York, Los Angeles, Seattle, and
                Miami. More regions are coming soon! Your server's location is currently chosen
                algorithmically, but you will be able to choose a region in the future.
              </p>
            </details>

            <details pyro-hash="storage" class="group" :open="$route.hash === '#storage'">
              <summary class="flex cursor-pointer items-center py-3 font-medium text-contrast">
                <span class="mr-2 transition-transform duration-200 group-open:rotate-90">
                  <RightArrowIcon />
                </span>
                Can I increase the storage on my server?
              </summary>
              <p class="m-0 !leading-[190%]">
                Yes, storage can be increased on your server at no additional cost. If you need more
                storage, reach out to Modrinth Support.
              </p>
            </details>

            <details pyro-hash="players" class="group" :open="$route.hash === '#players'">
              <summary class="flex cursor-pointer items-center py-3 font-medium text-contrast">
                <span class="mr-2 transition-transform duration-200 group-open:rotate-90">
                  <RightArrowIcon />
                </span>
                How fast are Modrinth Servers? How many players can they handle?
              </summary>
              <p class="m-0 !leading-[190%]">
                During the Modrinth "Emergency SMP" test, we had over 80 players on a server running
                on the Large plan. The server ran smoothly and was only limited by RAM. We're
                confident that Modrinth Servers can handle a large number of players, with any kind
                of modpack.
              </p>
            </details>
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
            isAtCapacity && !loggedOut
              ? "We are currently at capacity. Please try again later."
              : "There's a plan for everyone! Choose the one that fits your needs."
          }}
          <span class="font-bold">
            Servers are currently US only, in New York, Los Angeles, Seattle, and Miami. More
            regions coming soon!
          </span>
        </h2>

        <ul class="m-0 flex w-full flex-col gap-8 p-0 lg:flex-row">
          <li class="flex w-full flex-col gap-4 rounded-2xl bg-bg p-8 text-left lg:w-1/3">
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
              <p class="m-0">4 vCPUs</p>
              <div class="size-1.5 rounded-full bg-secondary opacity-25"></div>
              <p class="m-0">32 GB Storage</p>
            </div>
            <h2 class="m-0 text-3xl text-contrast">
              $12<span class="text-sm font-normal text-secondary">/month</span>
            </h2>
            <ButtonStyled color="blue" size="large">
              <NuxtLink
                v-if="!loggedOut && isSmallAtCapacity"
                :to="outOfStockUrl"
                target="_blank"
                class="!bg-highlight-blue !font-medium !text-blue"
              >
                Out of Stock
                <ExternalIcon class="!min-h-4 !min-w-4 !text-blue" />
              </NuxtLink>
              <button
                v-else
                class="!bg-highlight-blue !font-medium !text-blue"
                @click="selectProduct('small')"
              >
                Get Started
                <RightArrowIcon class="!min-h-4 !min-w-4 !text-blue" />
              </button>
            </ButtonStyled>
          </li>

          <li
            style="
              background: radial-gradient(
                86.12% 101.64% at 95.97% 94.07%,
                rgba(27, 217, 106, 0.23) 0%,
                rgba(14, 115, 56, 0.2) 100%
              );
              border: 1px solid rgba(12, 107, 52, 0.55);
              box-shadow: 0px 12px 38.1px rgba(27, 217, 106, 0.13);
            "
            class="flex w-full flex-col gap-4 rounded-2xl bg-bg p-8 text-left lg:w-1/3"
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
              <p class="m-0">6 vCPUs</p>
              <div class="size-1.5 rounded-full bg-secondary opacity-25"></div>
              <p class="m-0">48 GB Storage</p>
            </div>
            <h2 class="m-0 text-3xl text-contrast">
              $18<span class="text-sm font-normal text-secondary">/month</span>
            </h2>
            <ButtonStyled color="brand" size="large">
              <NuxtLink
                v-if="!loggedOut && isMediumAtCapacity"
                :to="outOfStockUrl"
                target="_blank"
                class="!bg-highlight-green !font-medium !text-green"
              >
                Out of Stock
                <ExternalIcon class="!min-h-4 !min-w-4 !text-green" />
              </NuxtLink>
              <button
                v-else
                class="!bg-highlight-green !font-medium !text-green"
                @click="selectProduct('medium')"
              >
                Get Started
                <RightArrowIcon class="!min-h-4 !min-w-4 !text-green" />
              </button>
            </ButtonStyled>
          </li>

          <li class="flex w-full flex-col gap-4 rounded-2xl bg-bg p-8 text-left lg:w-1/3">
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
              <p class="m-0">8 vCPUs</p>
              <div class="size-1.5 rounded-full bg-secondary opacity-25"></div>
              <p class="m-0">64 GB Storage</p>
            </div>
            <h2 class="m-0 text-3xl text-contrast">
              $24<span class="text-sm font-normal text-secondary">/month</span>
            </h2>
            <ButtonStyled color="brand" size="large">
              <NuxtLink
                v-if="!loggedOut && isLargeAtCapacity"
                :to="outOfStockUrl"
                target="_blank"
                class="!bg-highlight-purple !font-medium !text-purple"
              >
                Out of Stock
                <ExternalIcon class="!min-h-4 !min-w-4 !text-purple" />
              </NuxtLink>
              <button
                v-else
                class="!bg-highlight-purple !font-medium !text-purple"
                @click="selectProduct('large')"
              >
                Get Started
                <RightArrowIcon class="!min-h-4 !min-w-4 !text-purple" />
              </button>
            </ButtonStyled>
          </li>
        </ul>

        <div
          class="flex w-full flex-col items-start justify-between gap-4 rounded-2xl bg-bg p-8 text-left lg:flex-row lg:gap-0"
        >
          <div class="flex flex-col gap-4">
            <h1 class="m-0">Build your own</h1>
            <h2 class="m-0 text-base font-normal text-primary">
              If you're a more technical server administrator, you can pick your own RAM and storage
              options.
            </h2>
          </div>

          <div class="flex w-full flex-col-reverse gap-2 md:w-auto md:flex-col md:items-center">
            <ButtonStyled color="standard" size="large">
              <button class="w-full md:w-fit" @click="selectProduct('custom')">
                Build your own
                <RightArrowIcon class="!min-h-4 !min-w-4" />
              </button>
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
  TransferIcon,
  VersionIcon,
  ServerIcon,
} from "@modrinth/assets";
import { products } from "~/generated/state.json";
import LoaderIcon from "~/components/ui/servers/icons/LoaderIcon.vue";

const pyroProducts = products.filter((p) => p.metadata.type === "pyro");
const pyroPlanProducts = pyroProducts.filter(
  (p) => p.metadata.ram === 4096 || p.metadata.ram === 6144 || p.metadata.ram === 8192,
);
pyroPlanProducts.sort((a, b) => a.metadata.ram - b.metadata.ram);

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
const customServer = ref(false);
const showModal = ref(false);
const modalKey = ref(0);

const words = ["my-smp", "medieval-masters", "create-server", "mega-smp", "spookypack"];
const currentWordIndex = ref(0);
const currentText = ref("");
const isDeleting = ref(false);
const typingSpeed = 75;
const deletingSpeed = 25;
const pauseTime = 2000;

const loggedOut = computed(() => !auth.value.user);
const outOfStockUrl = "https://support.modrinth.com";

const { data: hasServers } = await useAsyncData("ServerListCountCheck", async () => {
  try {
    if (!auth.value.user) return false;
    const response = await usePyroFetch("servers");
    return response.servers && response.servers.length > 0;
  } catch {
    return false;
  }
});

async function fetchCapacityStatuses(customProduct = null) {
  try {
    const productsToCheck = customProduct?.metadata ? [customProduct] : pyroPlanProducts;
    const capacityChecks = productsToCheck.map((product) =>
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
    if (customProduct?.metadata) {
      return {
        custom: results[0],
      };
    } else {
      return {
        small: results[0],
        medium: results[1],
        large: results[2],
      };
    }
  } catch (error) {
    console.error("Error checking server capacities:", error);
    return {
      custom: { available: 0 },
      small: { available: 0 },
      medium: { available: 0 },
      large: { available: 0 },
    };
  }
}

const { data: capacityStatuses, refresh: refreshCapacity } = await useAsyncData(
  "ServerCapacityAll",
  fetchCapacityStatuses,
);

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

const scrollToFaq = () => {
  if (route.hash) {
    // where pyro-hash === route.hash
    const faq = document.querySelector(`[pyro-hash="${route.hash.slice(1)}"]`);
    if (faq) {
      faq.open = true;
      const top = faq.getBoundingClientRect().top;
      const offset = window.innerHeight / 2 - faq.clientHeight / 2;
      window.scrollTo({ top: window.scrollY + top - offset, behavior: "smooth" });
    }
  }
};

onMounted(scrollToFaq);

watch(() => route.hash, scrollToFaq);

const plans = {
  small: pyroPlanProducts?.[0],
  medium: pyroPlanProducts?.[1],
  large: pyroPlanProducts?.[2],
  custom: pyroProducts || [],
};

const selectProduct = async (product) => {
  if (loggedOut.value) {
    data.$router.push(`/auth/sign-in?redirect=${encodeURIComponent("/servers?plan=" + product)}`);
    return;
  }

  await refreshCapacity();
  if (isAtCapacity.value) {
    addNotification({
      group: "main",
      title: "Server Capacity Full",
      type: "error",
      text: "We are currently at capacity. Please try again later.",
    });
    return;
  }

  const selectedPlan = plans[product];
  if (!selectedPlan) return;

  if (
    (product === "custom" && !selectedPlan.length) ||
    (product !== "custom" && !selectedPlan.metadata)
  ) {
    addNotification({
      group: "main",
      title: "Invalid product",
      type: "error",
      text: "The selected product was found but lacks necessary data. Please contact support.",
    });
    return;
  }

  // required for the purchase modal
  if (!pyroProducts.metadata) {
    pyroProducts.metadata = {};
  }
  pyroProducts.metadata.type = "pyro";

  customServer.value = product === "custom";
  selectedProduct.value = selectedPlan;
  showModal.value = true;
  modalKey.value++;
  await nextTick();

  if (purchaseModal.value && purchaseModal.value.show) {
    purchaseModal.value.show();
  }
};

const planQuery = () => {
  if (route.query.plan) {
    document.getElementById("plan").scrollIntoView();
    selectProduct(route.query.plan);
  }
};

onMounted(() => {
  startTyping();
  planQuery();
});

watch(customer, (newCustomer) => {
  if (newCustomer) planQuery();
});

onMounted(() => {
  document.body.style.background = "var(--color-accent-contrast)";
  document.body.style.overflowX = "hidden !important";
  const layoutDiv = document.querySelector(".layout");
  if (layoutDiv) {
    layoutDiv.style.background = "var(--color-accent-contrast)";
  }
  fetchPaymentData();
});

onUnmounted(() => {
  document.body.style.background = "";
  document.body.style.overflowX = "";
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

.details-hide summary::-webkit-details-marker {
  display: none;
}
</style>
