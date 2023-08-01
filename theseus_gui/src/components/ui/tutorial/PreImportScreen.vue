<script setup>
import { Button, Card, ModrinthIcon } from 'omorphia'
import { ATLauncherIcon, PrismIcon } from '@/assets/external/index.js'

defineProps({
  nextPage: {
    type: Function,
    required: true,
  },
  prevPage: {
    type: Function,
    required: true,
  },
  importPage: {
    type: Function,
    required: true,
  },
})
</script>

<template>
  <Card class="import-card">
    <div class="base-ellipsis ellipsis-1" />
    <div class="base-ellipsis ellipsis-2" />
    <div class="base-ellipsis ellipsis-3" />
    <div class="base-ellipsis ellipsis-4" />
    <div class="logo">
      <ModrinthIcon />
    </div>
    <div class="launcher-stamp top-left">
      <ATLauncherIcon />
    </div>
    <div class="launcher-stamp top-right">
      <PrismIcon />
    </div>
    <div class="launcher-stamp bottom-left">
      <img src="@/assets/external/gdlauncher.png" alt="GDLauncher" />
    </div>
    <div class="launcher-stamp bottom-right">
      <img src="@/assets/external/multimc.webp" alt="MultiMC" />
    </div>
    <div class="info-section">
      <h2>Importing</h2>
      <div class="markdown-body">
        <p>
          You can import projects from other launchers by clicking below, or you can skip ahead.
        </p>
      </div>
      <div class="button-row">
        <Button class="transparent" @click="prevPage"> Back </Button>
        <Button color="primary" @click="importPage"> Import </Button>
        <Button class="transparent" @click="nextPage"> Next </Button>
      </div>
    </div>
  </Card>
</template>

<style scoped lang="scss">
.import-card {
  width: 40rem;
  height: 32rem;
  position: relative;
  overflow: hidden;
  padding: 0;
}

.base-ellipsis {
  position: absolute;
  left: 50%;
  border-radius: 100%;
  top: calc(var(--gap-xl) + 5rem);
  transform: translate(-50%, -50%);
  width: 100%;
  background-color: rgba(#1bd96a, 0.1);
}

.ellipsis-1 {
  width: 15rem;
  height: 15rem;
}

.ellipsis-2 {
  width: 30rem;
  height: 30rem;
}

.ellipsis-3 {
  width: 45rem;
  height: 45rem;
}

.logo {
  position: absolute;
  top: calc(var(--gap-xl) + 5rem);
  left: 50%;
  transform: translate(-50%, -50%);
  border-radius: 50%;
  background-color: var(--color-accent-contrast);
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 1rem;
  z-index: 1;
  width: 7rem;
  height: 7rem;

  svg {
    width: 100%;
    height: 100%;
  }
}

.launcher-stamp {
  position: absolute;
  width: 5rem;
  height: 5rem;
  background-color: var(--color-accent-contrast);
  border-radius: 50%;
  z-index: 1;
  opacity: 0.65;
  padding: var(--gap-lg);

  &.top-left {
    top: var(--gap-xl);
    left: 3rem;
  }

  &.top-right {
    top: var(--gap-xl);
    right: 3rem;
  }

  &.bottom-left {
    top: 12rem;
    left: 5.5rem;
  }

  &.bottom-right {
    top: 12rem;
    right: 5.5rem;
  }

  svg,
  img {
    width: 100%;
    height: 100%;
  }
}

.info-section {
  position: absolute;
  bottom: var(--gap-xl);
  left: 50%;
  width: 30rem;
  transform: translateX(-50%);
  padding: var(--gap-xl);
  display: flex;
  flex-direction: column;
  text-align: center;
  justify-content: center;
  align-items: center;
  gap: var(--gap-md);
  backdrop-filter: blur(1rem) brightness(0.4);
  -webkit-backdrop-filter: blur(1rem) brightness(0.4);
  border-radius: var(--radius-lg);

  h2 {
    margin: 0;
  }
}

.button-row {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  gap: var(--gap-md);
  width: 100%;
  align-content: center;

  .transparent {
    padding: var(--gap-sm) 0;
  }
}
</style>
