<script setup>
import { Button } from 'omorphia'
import { ref } from 'vue'
import { get, set } from '@/helpers/settings.js'
import mixpanel from 'mixpanel-browser'
import GalleryImage from '@/components/ui/tutorial/GalleryImage.vue'
import LoginCard from '@/components/ui/tutorial/LoginCard.vue'
import StickyTitleBar from '@/components/ui/tutorial/StickyTitleBar.vue'

const page = ref(1)

const props = defineProps({
  finish: {
    type: Function,
    default: () => {},
  },
})

const flow = ref('')

const nextPage = (newFlow) => {
  page.value++
  mixpanel.track('OnboardingPage', { page: page.value })

  if (newFlow) {
    flow.value = newFlow
  }
}

const prevPage = () => {
  page.value--
}

const finishOnboarding = async () => {
  mixpanel.track('OnboardingFinish')
  const settings = await get()
  settings.fully_onboarded = true
  await set(settings)
  props.finish()
}
</script>

<template>
  <div class="onboarding">
    <StickyTitleBar />
    <GalleryImage
      v-if="page === 1"
      :gallery="[
        {
          url: 'https://launcher-files.modrinth.com/onboarding/home.png',
          title: 'Discovery',
          subtitle: 'See the latest and greatest mods and modpacks to play with from Modrinth',
        },
        {
          url: 'https://launcher-files.modrinth.com/onboarding/discover.png',
          title: 'Profile Management',
          subtitle:
            'Play, manage and search through all the amazing profiles downloaded on your computer at any time, even offline!',
        },
      ]"
      logo
    >
      <Button color="primary" @click="nextPage"> Get started </Button>
    </GalleryImage>
    <LoginCard v-else-if="page === 2" :next-page="finishOnboarding" :prev-page="prevPage" />
  </div>
</template>

<style scoped lang="scss">
.sleek-primary {
  background-color: var(--color-brand-highlight);
  transition: all ease-in-out 0.1s;
}

.navigation-controls {
  flex-grow: 1;
  width: min-content;
}

.window-controls {
  z-index: 20;
  display: none;
  flex-direction: row;
  align-items: center;
  gap: 0.25rem;

  .titlebar-button {
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all ease-in-out 0.1s;
    background-color: var(--color-raised-bg);
    color: var(--color-base);

    &.close {
      &:hover,
      &:active {
        background-color: var(--color-red);
        color: var(--color-accent-contrast);
      }
    }

    &:hover,
    &:active {
      background-color: var(--color-button-bg);
      color: var(--color-contrast);
    }
  }
}

.container {
  --appbar-height: 3.25rem;
  --sidebar-width: 4.5rem;

  height: 100vh;
  display: flex;
  flex-direction: row;
  overflow: hidden;

  .view {
    width: calc(100% - var(--sidebar-width));

    .appbar {
      display: flex;
      align-items: center;
      background: var(--color-raised-bg);
      box-shadow: var(--shadow-inset-sm), var(--shadow-floating);
      padding: var(--gap-md);
      height: 3.25rem;
      gap: var(--gap-sm);
      user-select: none;
      -webkit-user-select: none;
    }

    .router-view {
      width: 100%;
      height: calc(100% - 3.125rem);
      overflow: auto;
      overflow-x: hidden;
      background-color: var(--color-bg);
    }
  }
}

.nav-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: space-between;
  height: 100%;
  background-color: var(--color-raised-bg);
  box-shadow: var(--shadow-inset-sm), var(--shadow-floating);
  padding: var(--gap-md);
  width: var(--sidebar-width);
  max-width: var(--sidebar-width);
  min-width: var(--sidebar-width);

  --sidebar-width: 4.5rem;
}

.pages-list {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: flex-start;
  width: 100%;
  gap: 0.5rem;

  .btn {
    background-color: var(--color-raised-bg);
    height: 3rem !important;
    width: 3rem !important;
    padding: 0.75rem;
    border-radius: var(--radius-md);
    box-shadow: none;

    svg {
      width: 1.5rem !important;
      height: 1.5rem !important;
      max-width: 1.5rem !important;
      max-height: 1.5rem !important;
    }

    &.active {
      background-color: var(--color-button-bg);
      box-shadow: var(--shadow-floating);
    }

    &.sleek-primary {
      background-color: var(--color-brand-highlight);
      transition: all ease-in-out 0.1s;
    }
  }
}

.nav-section {
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  align-items: center;
  width: 100%;
  height: 100%;
  gap: 1rem;
}

.sticky-tip {
  position: absolute;
  bottom: 1rem;
  right: 1rem;
  z-index: 10;
}

.intro-card {
  display: flex;
  flex-direction: column;
  padding: var(--gap-xl);

  .app-logo {
    width: 100%;
    height: auto;
    display: block;
    margin: auto;
  }

  p {
    color: var(--color-contrast);
    text-align: left;
    width: 100%;
  }

  .actions {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: flex-end;
    gap: var(--gap-sm);
  }
}

.final-tip {
  position: absolute;
  bottom: 50%;
  right: 50%;
  transform: translate(50%, 50%);
  z-index: 10;
}

.onboarding {
  background:
    top linear-gradient(0deg, #31375f, rgba(8, 14, 55, 0)),
    url(https://cdn.modrinth.com/landing-new/landing-lower.webp);
  background-size: cover;
  height: 100vh;
  min-height: 100vh;
  max-height: 100vh;
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--gap-xl);
  padding-top: calc(2.5rem + var(--gap-lg));
}

.first-tip {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 10;
}

.whole-page-shadow {
  position: absolute;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100%;
  backdrop-filter: brightness(0.5);
  -webkit-backdrop-filter: brightness(0.5);
  z-index: 9;
}
</style>
