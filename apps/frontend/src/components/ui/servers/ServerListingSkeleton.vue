<template>
  <div
    class="flex flex-row items-center overflow-x-hidden rounded-3xl bg-bg-raised p-4 transition-transform duration-100 active:scale-95"
  >
    <img
      no-shadow
      size="lg"
      alt="Server Icon"
      class="size-[96px] rounded-xl bg-bg-raised"
      src="~/assets/images/servers/minecraft_server_icon.png"
    />
    <div class="ml-8 flex flex-col gap-2.5">
      <div class="flex flex-col gap-2 md:flex-row md:items-center">
        <h2 class="m-0 text-xl font-bold text-[var(--color-contrast)]">Loading...</h2>
      </div>
    </div>
  </div>
</template>
