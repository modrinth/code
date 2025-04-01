<template>
  <NewModal ref="modal" header="Build your pizza">
    <div class="flex flex-col gap-4">
      <!-- Pizza Name -->
      <div class="flex flex-col gap-2">
        <label for="pizza-name">
          <span class="text-lg font-semibold text-contrast">
            Pizza Name
            <span class="text-brand-red">*</span>
          </span>
        </label>
        <input
          id="pizza-name"
          v-model="name"
          type="text"
          maxlength="64"
          placeholder="Ex: Creeper Crust Deluxe"
          autocomplete="off"
        />
      </div>

      <!-- Crust Type Dropdown -->
      <div class="flex flex-col gap-2">
        <label for="crust">
          <span class="text-lg font-semibold text-contrast">
            Crust Type
            <span class="text-brand-red">*</span>
          </span>
        </label>
        <DropdownSelect
          id="crust"
          v-model="crust"
          :options="crustOptions"
          :display-name="(x: any) => x.display"
          name="Crust Type"
        />
      </div>

      <!-- Toppings (text only display) -->
      <div class="flex flex-col gap-2">
        <span class="text-lg font-semibold text-contrast">Toppings</span>
        <span
          >Your preselected topping choice. Our advanced AI determined these top 3 toppings for
          you.</span
        >
        <div>
          {{ toppingOptions.join(", ") }}
        </div>
      </div>

      <!-- Pizza Description -->
      <div class="flex flex-col gap-2">
        <label for="description" class="flex flex-col gap-1">
          <span class="text-lg font-semibold text-contrast"> Description </span>
          <span> Describe your pizza (mods, flavor, lore...) </span>
        </label>
        <div class="textarea-wrapper">
          <textarea id="description" v-model="description" maxlength="256" />
        </div>
      </div>

      <!-- If the order contains pineapple, show a warning -->
      <div v-if="toppingOptions.includes('Pineapple Ore')" class="flex flex-col gap-2">
        <span class="text-lg font-semibold text-contrast">Order validation</span>
        <div class="flex flex-row gap-2">
          <span class="text-yellow-500"
            ><b>Warning!</b> Pineapple detected. This may cause unknown side effects.</span
          >
        </div>
      </div>

      <!-- If no pineapple, say everything is okay -->
      <div v-else class="flex flex-col gap-2">
        <span class="text-lg font-semibold text-contrast">Order validation</span>
        <span class="text-green-500"
          ><b>Success!</b> No pineapple detected. Your order is safe.</span
        >
      </div>

      <!-- Action Buttons -->
      <div class="flex gap-2">
        <ButtonStyled color="brand">
          <button @click="orderPizza">
            <PlusIcon aria-hidden="true" />
            Order Pizza
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button @click="cancel">
            <XIcon aria-hidden="true" />
            Cancel
          </button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>

  <div
    v-if="isPreapring"
    class="fixed left-0 top-0 flex h-full w-full items-center justify-center bg-black bg-opacity-50"
  >
    <div class="rounded-lg bg-zinc-800 p-8 shadow-lg">
      <h2 class="mb-4 text-2xl font-semibold">Your pizza is being prepared...</h2>
      <p class="text-lg">
        Your chef, <span class="font-semibold">{{ whoIsPreparing }}</span
        >, is preparing your pizza. Please wait warmly.
      </p>
      <p class="mt-2 text-lg">
        Estimated time left: <span class="font-semibold">{{ preparingTime }} seconds</span>
      </p>
    </div>
  </div>

  <main class="flex flex-1 flex-col items-center justify-center px-6 py-16 text-center">
    <img class="mb-4 h-auto w-48 md:w-64" src="https://i.hep.gg/RRLj-WEql" alt="Modrinth Pizza" />

    <h2 class="mb-4 text-4xl font-bold md:text-5xl">Welcome to Modrinth Pizza</h2>
    <p class="mb-6 max-w-xl text-lg">
      Say hello to <span class="text-green-400 font-semibold">Modrinth Pizza</span> — the world's
      first modded pizza delivery service. Pick your loader (Forge, Fabric, or Quilt crust), and get
      delicious slices shipped straight to your server.
    </p>

    <p class="mb-8 max-w-lg text-base">
      Compatible with all Minecraft flavors, auto-updates via <code>cheese.toml</code>, and is
      <span class="text-green-400 font-semibold">fully open sauce</span>.
    </p>

    <ButtonStyled color="brand">
      <button @click="modal.show">
        <svg class="h-5 w-5 fill-current">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="1.5"
            stroke="currentColor"
            class="size-6"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M2.25 3h1.386c.51 0 .955.343 1.087.835l.383 1.437M7.5 14.25a3 3 0 0 0-3 3h15.75m-12.75-3h11.218c1.121-2.3 2.1-4.684 2.924-7.138a60.114 60.114 0 0 0-16.536-1.84M7.5 14.25 5.106 5.272M6 20.25a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Zm12.75 0a.75.75 0 1 1-1.5 0 .75.75 0 0 1 1.5 0Z"
            />
          </svg>
        </svg>
        Order now
      </button>
    </ButtonStyled>

    <p class="mt-4 max-w-lg text-sm text-gray-500">
      Disclaimer: Modrinth Pizza is not responsible for any mod conflicts, crashes, or server
      outages caused by our pizza. Eat at your own risk. By ordering, you agree to this and to allow
      us to collect your data to sell to the highest bidder. Bon appétit! 🍕
    </p>
  </main>
</template>
<script setup lang="ts">
import { NewModal, ButtonStyled, DropdownSelect } from "@modrinth/ui";
import { XIcon, PlusIcon } from "@modrinth/assets";

const modal = ref();
const name = ref("");
const crust = ref({ value: "fabric", display: "Fabric" });
const toppings = ref([]);
const description = ref("");
const pineapple_opinions = ref([]);

const crustOptions = [
  { value: "fabric", display: "Fabric" },
  { value: "forge", display: "Forge" },
  { value: "quilt", display: "Quilt" },
];

const toppingOptions = ref([
  "Pineapple Ore",
  "Redstone Pepperoni",
  "Netherite Bacon",
  "Glow Lichen",
  "Ender Anchovies",
]);

const modrinthTeamMembers = [
  "Tanner",
  "Ben",
  "coolbot",
  "François",
  "Jai",
  "Prospector",
  "Kchem",
  "Jade",
  "Jasmine",
  "Michael",
  "Frobert",
];

const isPreapring = ref(false);
const whoIsPreparing = ref(
  modrinthTeamMembers[Math.floor(Math.random() * modrinthTeamMembers.length)],
);

// Derive the preparing time from the index of whoIsPreparing
const preparingTime = ref(5 + modrinthTeamMembers.indexOf(whoIsPreparing.value) * 2);

function cancel() {
  modal.value.hide();
}

function show(event: MouseEvent) {
  name.value = "";
  crust.value = crustOptions[0];
  toppings.value = [];
  description.value = "";
  modal.value.show(event);
}

function orderPizza() {
  // If any value is empty, show an error notification
  if (!name.value || !crust.value || !description.value) {
    addNotification({
      title: "Error!",
      text: "Please fill out all required fields.",
      type: "error",
    });
    return;
  }

  // Make sure the math problem is solved, if incorrect, redirect to rickroll
  if (toppingOptions.value.includes("Pineapple Ore") && pineapple_opinions.value.length !== 1) {
    // Roll a random number between 1 and 100, if above 50, redirect to rickroll
    if (Math.floor(Math.random() * 100) > 50) {
      addNotification({
        title: "Warning!",
        text: "pizza_create: create(0) failed: Pineapple detected. Recovery process initiated.",
        type: "error",
      });

      setTimeout(() => {
        window.location.href = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
      }, 2000);
      return;
    }
  }

  modal.value.hide();

  // Show the preparing modal
  isPreapring.value = true;
  addNotification({
    title: "Success!",
    text: "Order successfully placed. Your pizza is being prepared.",
    type: "success",
  });

  const inter = setInterval(() => {
    preparingTime.value--;
    if (preparingTime.value === 0) {
      addNotification({
        title: "Success!",
        text: "Your pizza is ready! 🍕 You will be redirected to the delivery page.",
        type: "success",
      });

      clearInterval(inter);

      // Redirect to funny video :)
      setTimeout(() => {
        window.location.href = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
      }, 2000);
    }
  }, 1000);
}

defineExpose({
  show,
});

// on page load, shuffle the toppings, slice to top 3, and set them as preselected
onMounted(() => {
  const shuffledArray = [...toppingOptions.value].sort(() => 0.5 - Math.random());
  // Replace the contents of toppingOptions with the shuffled array
  toppingOptions.value = shuffledArray.slice(0, 3);
});
</script>
