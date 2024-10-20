<template>
  <div
    class="grid grid-cols-1 gap-x-8 gap-y-6 border-0 border-b border-solid border-button-bg pb-6 lg:grid-cols-[1fr_auto]"
  >
    <div class="flex gap-4">
      <slot name="icon" />
      <div class="flex flex-col gap-1">
        <div class="flex flex-wrap items-center gap-2">
          <h1 class="m-0 text-2xl font-extrabold leading-none text-contrast">
            <slot name="title" />
          </h1>
          <slot name="title-suffix" />
        </div>
        <p class="m-0 line-clamp-2 max-w-[40rem]">
          <slot name="summary" />
        </p>
        <div class="mt-auto flex flex-wrap gap-4">
          <slot name="stats" />
        </div>
      </div>
    </div>
    <div class="flex flex-wrap gap-2 items-center">
      <slot name="actions" />
    </div>
  </div>
</template>
