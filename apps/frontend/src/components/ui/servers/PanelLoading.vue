<template>
  <div
    aria-hidden="true"
    class="experimental-styles-within pointer-events-none mx-auto flex min-h-screen w-full max-w-[1280px] select-none flex-col gap-6 px-4 sm:px-6"
  >
    <div class="flex flex-row items-center gap-6 pt-4">
      <div class="h-[144px] w-[144px] animate-pulse rounded-2xl bg-bg-raised"></div>
      <div class="flex flex-col gap-4">
        <div class="-mb-2 flex shrink-0 flex-row items-center gap-1">
          <a href="/servers/manage" class="breadcrumb goto-link flex w-fit items-center">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14.414"
              height="12.162"
              viewBox="0 0 14.414 12.162"
            >
              <path
                fill="none"
                stroke="currentColor"
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M6.081 10.747 1.414 6.081m0 0 4.667-4.667M1.414 6.081h12"
              ></path>
            </svg>
            All servers
          </a>
        </div>
        <span class="h-10 w-[200px] animate-pulse rounded-full bg-bg-raised"></span>
        <div class="flex flex-row items-center gap-4 text-[var(--color-text-secondary)]">
          <div class="flex flex-row items-center gap-2">
            <img
              src="/_nuxt/assets/images/games/minecraft.png"
              alt="Minecraft Logo"
              class="size-5"
            /><span class="text-sm font-semibold">Minecraft</span>
          </div>
          <div class="flex flex-row items-center gap-4">
            <div class="experimental-styles-within h-6 w-0.5 bg-button-border"></div>
            <div class="flex animate-pulse flex-row items-center gap-2">
              <div class="h-4 w-4 rounded-full bg-bg-raised"></div>
              <span class="h-4 w-[100px] rounded-full bg-bg-raised"></span>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div class="flex flex-row items-center justify-between">
      <nav
        class="relative flex w-fit overflow-clip rounded-full bg-bg-raised p-1 text-sm font-bold"
      >
        <div
          class="router-link-active router-link-exact-active button-animation flex flex-row items-center gap-2 px-4 py-2 text-brand focus:rounded-full"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            fill="none"
            stroke="currentColor"
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            viewBox="0 0 24 24"
            class="size-5"
          >
            <path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>
            <path d="M9 22V12h6v10"></path></svg
          ><span>Overview</span>
        </div>
        <div class="button-animation flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="currentColor"
            viewBox="0 0 20 20"
            class="size-5"
          >
            <path
              d="M10.362 1.093a.75.75 0 0 0-.724 0L2.523 5.018 10 9.143l7.477-4.125zM18 6.443l-7.25 4v8.25l6.862-3.786A.75.75 0 0 0 18 14.25zm-8.75 12.25v-8.25l-7.25-4v7.807a.75.75 0 0 0 .388.657z"
            ></path></svg
          ><span>Content</span>
        </div>
        <div class="button-animation flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="currentColor"
            viewBox="0 0 20 20"
            class="size-5"
          >
            <path
              d="M1 12.5A4.5 4.5 0 0 0 5.5 17H15a4 4 0 0 0 1.866-7.539 3.504 3.504 0 0 0-4.504-4.272A4.5 4.5 0 0 0 4.06 8.235 4.5 4.5 0 0 0 1 12.5"
            ></path></svg
          ><span>Backups</span>
        </div>
        <div class="button-animation flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="currentColor"
            viewBox="0 0 20 20"
            class="size-5"
          >
            <path
              fill-rule="evenodd"
              d="M7.84 1.804A1 1 0 0 1 8.82 1h2.36a1 1 0 0 1 .98.804l.331 1.652a7 7 0 0 1 1.929 1.115l1.598-.54a1 1 0 0 1 1.186.447l1.18 2.044a1 1 0 0 1-.205 1.251l-1.267 1.113a7 7 0 0 1 0 2.228l1.267 1.113a1 1 0 0 1 .206 1.25l-1.18 2.045a1 1 0 0 1-1.187.447l-1.598-.54a7 7 0 0 1-1.929 1.115l-.33 1.652a1 1 0 0 1-.98.804H8.82a1 1 0 0 1-.98-.804l-.331-1.652a7 7 0 0 1-1.929-1.115l-1.598.54a1 1 0 0 1-1.186-.447l-1.18-2.044a1 1 0 0 1 .205-1.251l1.267-1.114a7 7 0 0 1 0-2.227L1.821 7.773a1 1 0 0 1-.206-1.25l1.18-2.045a1 1 0 0 1 1.187-.447l1.598.54a7 7 0 0 1 1.93-1.115zM10 13a3 3 0 1 0 0-6 3 3 0 0 0 0 6"
              clip-rule="evenodd"
            ></path></svg
          ><span>Options</span>
        </div>
        <div
          class="pointer-events-none absolute h-[calc(100%-0.5rem)] overflow-hidden rounded-full bg-brand p-1 transition-all"
          aria-hidden="true"
          style="left: 4px; top: 4px; width: 125px; opacity: 0.25"
        ></div>
      </nav>
      <div class="flex flex-row gap-2">
        <div class="btn btn-transparent">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            fill="none"
            stroke="currentColor"
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            class="lucide lucide-copy"
            viewBox="0 0 24 24"
          >
            <rect width="14" height="14" x="8" y="8" rx="2" ry="2"></rect>
            <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"></path>
          </svg>
          Copy IP
        </div>
        <div class="btn btn-primary">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            fill="none"
            stroke="currentColor"
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            viewBox="0 0 24 24"
          >
            <path d="m5 3 14 9-14 9z"></path>
          </svg>
          Play
        </div>
      </div>
    </div>
  </div>
</template>
