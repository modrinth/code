<template>
  <div data-pyro class="servers-hero relative -mt-48 h-full min-h-screen py-48 md:-mt-20 md:py-32">
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
    <img
      src="~/assets/images/games/maze.png"
      alt=""
      aria-hidden="true"
      class="pointer-events-none absolute inset-0 hidden h-full max-h-[1080px] w-screen scale-125 select-none opacity-50 sm:visible"
      style="mask-image: linear-gradient(black, transparent 80%)"
    />

    <section class="relative mx-auto flex max-w-7xl flex-col px-3">
      <div class="flex w-full flex-col gap-10">
        <div
          class="relative w-fit rounded-full bg-highlight-green px-3 py-1 text-sm font-bold text-brand backdrop-blur-lg"
        >
          Experimental
        </div>
        <h1 class="relative m-0 max-w-2xl text-4xl font-extrabold leading-[120%] md:text-7xl">
          Play together on Modrinth Servers
        </h1>
        <h2
          class="relative m-0 max-w-2xl text-base font-normal leading-[155%] text-secondary md:text-[1.2rem]"
        >
          Start your own Minecraft server directly on Modrinth. Play your favorite mods, plugins,
          and datapacks with friends — without the hassle of setup.
        </h2>
        <div class="relative flex w-fit flex-row gap-8">
          <ButtonStyled color="brand" size="large">
            <nuxt-link class="w-fit" to="#plan"> Start your server </nuxt-link>
          </ButtonStyled>
          <UiServersPoweredByPyro class="!mt-0" />
        </div>

        <div
          class="pointer-events-none relative flex h-full w-full flex-row items-end gap-24 md:-mt-24"
        >
          <div
            class="absolute left-0 right-0 top-0 -mt-24 max-h-[80%] overflow-hidden sm:mt-0"
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
          Choose from the thousands of modpacks on Modrinth or create your own. Invite your friends
          when you're ready to play.
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
            <h2 class="m-0 text-lg font-bold">Easily join your friends</h2>
            <h3 class="m-0 text-base font-normal text-secondary">
              Your servers are sharable with just one link. Easily invite your friends to join your
              server. Modrinth takes care of installing the mods.
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
      <div class="faded-brand-line absolute top-0 h-[1px] w-full"></div>
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
                With 15 backup slots stored off-site with Backblaze, your server is always safe.
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
            class="relative flex flex-col gap-4 overflow-hidden rounded-2xl p-6 text-left backdrop-blur-xl md:p-12"
          >
            <h2 class="m-0 text-lg font-bold">Easy to use file manager</h2>
            <h3 class="m-0 text-base font-normal">
              Manage, search, and upload files directly to your server with ease.
            </h3>

            <img
              src="~/assets/images/games/content-hero-fixed.png"
              alt=""
              class="absolute -bottom-12 -right-[15%] max-w-2xl rounded-2xl bg-brand p-4"
            />
            <div
              aria-hidden="true"
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
            Reach out to the Modrinth team for help with your server at any time.
          </h3>
        </div>
      </div>
    </section>

    <section
      id="plan"
      class="relative mt-24 flex flex-col bg-[radial-gradient(65%_50%_at_50%_-10%,var(--color-brand-highlight)_0%,var(--color-accent-contrast)_100%)] px-3 pt-24 md:mt-48 md:pt-48"
    >
      <div class="faded-brand-line absolute top-0 h-[1px] w-full"></div>
      <div class="mx-auto flex w-full max-w-7xl flex-col items-center gap-8 text-center">
        <h1 class="relative m-0 text-4xl leading-[120%] md:text-7xl">
          Start your server on Modrinth
        </h1>
        <h2
          class="relative m-0 max-w-xl text-base font-normal leading-[155%] text-secondary md:text-[18px]"
        >
          There's a plan for everyone. Choose the one that fits your needs.
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
            <div class="flex flex-row items-center gap-3">
              <p class="m-0">4 GB RAM</p>
              <div class="size-1.5 rounded-full bg-secondary opacity-25"></div>
              <p class="m-0">32 GB Storage</p>
            </div>
            <h2 class="m-0 text-3xl text-contrast">
              $12<span class="text-sm font-normal text-secondary">/month</span>
            </h2>
            <ButtonStyled color="blue" size="large">
              <button
                class="!bg-highlight-blue !font-medium !text-blue"
                @click="selectProduct(pyroProducts[0])"
              >
                Get Started
                <RightArrowIcon class="!min-h-4 !min-w-4" />
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
              border: 1px solid rgba(27, 217, 106, 0.07);
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
            <div class="flex flex-row items-center gap-3">
              <p class="m-0">6 GB RAM</p>
              <div class="size-1.5 rounded-full bg-secondary opacity-25"></div>
              <p class="m-0">48 GB Storage</p>
            </div>
            <h2 class="m-0 text-3xl text-contrast">
              $18<span class="text-sm font-normal text-secondary">/month</span>
            </h2>
            <ButtonStyled color="brand" size="large">
              <button class="shadow-xl" @click="selectProduct(pyroProducts[1])">
                Get Started
                <RightArrowIcon class="!min-h-4 !min-w-4" />
              </button>
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
            <div class="flex flex-row items-center gap-3">
              <p class="m-0">8 GB RAM</p>
              <div class="size-1.5 rounded-full bg-secondary opacity-25"></div>
              <p class="m-0">64 GB Storage</p>
            </div>
            <h2 class="m-0 text-3xl text-contrast">
              $24<span class="text-sm font-normal text-secondary">/month</span>
            </h2>
            <ButtonStyled color="purple" size="large">
              <button
                class="!bg-highlight-purple !font-medium !text-purple"
                @click="selectProduct(pyroProducts[2])"
              >
                Get Started
                <RightArrowIcon class="!min-h-4 !min-w-4" />
              </button>
            </ButtonStyled>
          </li>
        </ul>

        <div
          class="flex w-full flex-row items-start justify-between rounded-2xl bg-bg p-8 text-left"
        >
          <div class="flex flex-col gap-4">
            <h1 class="m-0">Build your own</h1>
            <h2 class="m-0 text-base font-normal">
              If you're a more technical server administator, you can pick your own RAM and storage
              options.
            </h2>
          </div>

          <div class="flex flex-col items-center gap-2">
            <ButtonStyled color="standard" size="large">
              <NuxtLink to="/servers/custom" class="w-fit">
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
import { ref, onMounted, watch, nextTick } from "vue";
import { ButtonStyled, PurchaseModal } from "@modrinth/ui";
import { RightArrowIcon } from "@modrinth/assets";
import { products } from "~/generated/state.json";

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

const pyroProducts = products.filter((p) => p.metadata.type === "pyro");
pyroProducts.sort((a, b) => a.metadata.ram - b.metadata.ram);

const selectProduct = async (product) => {
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
const openPurchaseModal = () => {
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
