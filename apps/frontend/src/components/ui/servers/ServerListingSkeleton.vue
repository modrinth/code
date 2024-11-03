<template>
  <div class="flex flex-row items-center gap-8 overflow-x-hidden rounded-3xl bg-bg-raised p-4">
    <div class="relative grid place-content-center">
      <img
        no-shadow
        size="lg"
        alt="Server Icon"
        class="size-[96px] rounded-xl bg-bg-raised opacity-50"
        src="~/assets/images/servers/minecraft_server_icon.png"
      />
      <div class="absolute inset-0 grid place-content-center">
        <UiServersIconsLoadingIcon class="size-8 animate-spin text-contrast" />
      </div>
    </div>
    <div class="flex flex-col gap-4">
      <h2 class="m-0 text-contrast">Your new server is being prepared.</h2>
      <p class="m-0">It'll appear here once it's ready.</p>
    </div>
  </div>
</template>
