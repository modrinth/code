<script setup>
import {Button, HomeIcon, SearchIcon, LibraryIcon, PlusIcon, SettingsIcon, XIcon, Notifications, LogOutIcon} from "omorphia";
import OnboardingModal from "@/components/ui/tutorial/OnboardingModal.vue";
import {appWindow} from "@tauri-apps/api/window";
import {saveWindowState, StateFlags} from "tauri-plugin-window-state-api";
import Breadcrumbs from "@/components/ui/Breadcrumbs.vue";
import FakeAppBar from "@/components/ui/tutorial/FakeAppBar.vue";
import FakeAccountsCard from "@/components/ui/tutorial/FakeAccountsCard.vue";
import {MinimizeIcon, MaximizeIcon} from "@/assets/icons";
import ModrinthLoadingIndicator from "@/components/modrinth-loading-indicator.js";
import FakeSearch from "@/components/ui/tutorial/FakeSearch.vue";
import FakeGridDisplay from "@/components/ui/tutorial/FakeGridDisplay.vue";
import FakeRowDisplay from "@/components/ui/tutorial/FakeRowDisplay.vue";
import {ref} from "vue";
import {window} from "@tauri-apps/api";
import TutorialTip from "@/components/ui/tutorial/TutorialTip.vue";
import FakeSettings from "@/components/ui/tutorial/FakeSettings.vue";
import {get, set} from "@/helpers/settings.js";
import mixpanel from "mixpanel-browser";

const phase = ref(0);

const props = defineProps({
  finish: {
    type: Function,
    default: () => {}
  }
})

const nextPhase = () => {
  phase.value++;
  mixpanel.track('TutorialPhase', { page: phase.value })
}

const finishOnboarding = async () => {
  mixpanel.track('OnboardingFinish')
  const settings = await get();
  settings.onboarded = true;
  await set(settings);
  props.finish();
}
</script>

<template>
  <suspense>
    <OnboardingModal/>
  </suspense>
  <div class="container">
    <div class="nav-container expanded">
      <div class="nav-section">
        <FakeAccountsCard :show-demo="phase === 3">
          <TutorialTip
            :progress-function="nextPhase"
            :progress="phase"
            title="Signing in"
            description="The Modrinth App uses your Modrinth account to allow you to launch Minecraft. You can sign in with your Modrinth account here, and switch between multiple accounts."
          />
        </FakeAccountsCard>
        <div class="pages-list">
          <div class="btn expanded-button" :class="{'active': phase < 4}">
            <HomeIcon />
            Home
          </div>
          <div class="btn expanded-button" :class="{'active': phase === 4 || phase === 5}">
            <SearchIcon />
            Browse
          </div>
          <div
            class="btn expanded-button"
            :class="{
              'active': phase === 6 || phase === 7,
              'highlighted': phase === 6
            }"
          >
            <LibraryIcon />
            Library
          </div>
        </div>
      </div>
      <div class="settings pages-list">
        <Button class="active expanded-button" @click="() => phase = 1">
          <LogOutIcon />
          Exit Tutorial
        </Button>
        <Button class="sleek-primary expanded-button">
          <PlusIcon />
          New instance
        </Button>
        <div class="btn expanded-button" :class="{'active': phase === 8, 'highlighted': phase === 8}">
          <SettingsIcon />
          Settings
        </div>
      </div>
    </div>
    <div class="view expanded">
      <div data-tauri-drag-region class="appbar">
        <section class="navigation-controls">
          <Breadcrumbs data-tauri-drag-region />
        </section>
        <section class="mod-stats">
          <FakeAppBar :show-running="phase === 7" :show-download="phase === 5">
            <template #running>
              <TutorialTip
                :progress-function="nextPhase"
                :progress="phase"
                title="Playing modpacks"
                description="When you launch a modpack, you can manage it directly in the title bar here. You can stop the modpack, view the logs, and see all currently running packs."
              />
            </template>
            <template #download>
              <TutorialTip
                :progress-function="nextPhase"
                :progress="phase"
                title="Installing modpacks"
                description="When you download a modpack, Modrinth App will automatically install it for you. You can view the progress of the installation here."
              />
            </template>
          </FakeAppBar>
        </section>
        <section class="window-controls">
          <Button class="titlebar-button" icon-only @click="() => appWindow.minimize()">
            <MinimizeIcon />
          </Button>
          <Button class="titlebar-button" icon-only @click="() => appWindow.toggleMaximize()">
            <MaximizeIcon />
          </Button>
          <Button
            class="titlebar-button close"
            icon-only
            @click="
                () => {
                  saveWindowState(StateFlags.ALL)
                  window.getCurrent().close()
                }
              "
          >
            <XIcon />
          </Button>
        </section>
      </div>
      <div class="router-view">
        <ModrinthLoadingIndicator
          offset-height="var(--appbar-height)"
          offset-width="var(--sidebar-width)"
        />
        <Notifications ref="notificationsWrapper" />
        <FakeRowDisplay v-if="phase < 4" :show-instance="phase===2"/>
        <FakeGridDisplay v-if="phase === 6 || phase === 7" :show-instances="phase===6" />
        <suspense>
          <FakeSearch v-if="phase === 4 || phase === 5" :show-search="phase===4"/>
        </suspense>
        <FakeSettings v-if="phase === 8" />
      </div>
    </div>
    <TutorialTip
      v-if="phase === 1"
      class="sticky-tip"
      :progress-function="nextPhase"
      :progress="phase"
      title="Welcome to the Modrinth App!"
      description="This is the Modrinth App, a desktop client for Modrinth. This tutorial will show you how to use the app."
    />
    <TutorialTip
      v-if="phase === 2"
      class="sticky-tip"
      :progress-function="nextPhase"
      :progress="phase"
      title="Home page"
      description="This is the home page. Here you can see all the latest modpacks, mods, and other content on Modrinth. You can also see a few of your installed modpacks here."
    />
    <TutorialTip
      v-if="phase === 4"
      class="sticky-tip"
      :progress-function="nextPhase"
      :progress="phase"
      title="Searching for content"
      description="You can search for content on Modrinth by navigating to the search page. You can search for mods, modpacks, and more, and install them directly from here."
    />
    <TutorialTip
      v-if="phase === 6"
      class="sticky-tip"
      :progress-function="nextPhase"
      :progress="phase"
      title="Modpack library"
      description="You can view all your installed modpacks in the library. You can launch any modpack from here, or click the card to view more information about it."
    />
    <TutorialTip
      v-if="phase === 8"
      class="sticky-tip"
      :progress-function="nextPhase"
      :progress="phase"
      title="Settings"
      description="You can view and change the settings for the Modrinth App here. You can change the appearance, set and download new Java versions, and more."
    />
    <TutorialTip
      v-if="phase === 9"
      class="final-tip"
      :progress-function="finishOnboarding"
      :progress="phase"
      title="Enter the Modrinth App!"
      description="That's it! You're ready to use the Modrinth App. If you need help, you can always join our discord server!"
    />
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
      &:hover, &:active {
        background-color: var(--color-red);
        color: var(--color-accent-contrast);
      }
    }

    &:hover, &:active {
      background-color: var(--color-button-bg);
      color: var(--color-contrast);
    }
  }
}

.container {
  --appbar-height: 3.25rem;
  --sidebar-width: 13rem;

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

  --sidebar-width: 13rem;
}

.pages-list {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: flex-start;
  width: 100%;
  gap: 0.5rem;

  .btn {
    width: 100%;
    padding: var(--gap-md) var(--gap-lg);
    background-color: var(--color-raised-bg);
    box-shadow: none;

    &.active {
      background-color: var(--color-button-bg);
      box-shadow: var(--shadow-floating);
    }

    &.sleek-primary {
      background-color: var(--color-brand-highlight);
      transition: all ease-in-out 0.1s;
    }

    &.sleek-exit {
      background-color: var(--color-red);
      color: var(--color-accent-contrast);
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

.final-tip {
  position: absolute;
  bottom: 50%;
  right: 50%;
  transform: translate(50%, 50%);
  z-index: 10;
}
</style>
