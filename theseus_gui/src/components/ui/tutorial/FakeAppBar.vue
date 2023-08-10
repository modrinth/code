<template>
  <div class="action-groups">
    <Button color="danger" outline @click="exit">
      <LogOutIcon />
      Exit tutorial
    </Button>
    <Button v-if="showDownload" ref="infoButton" icon-only class="icon-button show-card-icon">
      <DownloadIcon />
    </Button>
    <div v-if="showRunning" class="status highlighted">
      <span class="circle running" />
      <div ref="profileButton" class="running-text">Example Modpack</div>
      <Button v-tooltip="'Stop instance'" icon-only class="icon-button stop">
        <StopCircleIcon />
      </Button>
      <Button v-tooltip="'View logs'" icon-only class="icon-button">
        <TerminalSquareIcon />
      </Button>
    </div>
    <div v-else class="status">
      <span class="circle stopped" />
      <span class="running-text"> No running instances </span>
    </div>
  </div>
  <transition name="download">
    <div v-if="showDownload" class="info-section">
      <Card ref="card" class="highlighted info-card">
        <h3 class="info-title">New Modpack</h3>
        <ProgressBar :progress="50" />
        <div class="row">50% Downloading modpack</div>
      </Card>
      <slot name="download" />
    </div>
  </transition>
  <transition name="running">
    <div v-if="showRunning" class="info-section">
      <slot name="running" />
    </div>
  </transition>
</template>

<script setup>
import {
  Button,
  DownloadIcon,
  Card,
  StopCircleIcon,
  TerminalSquareIcon,
  LogOutIcon,
} from 'omorphia'
import ProgressBar from '@/components/ui/ProgressBar.vue'

defineProps({
  showDownload: {
    type: Boolean,
    default: false,
  },
  showRunning: {
    type: Boolean,
    default: false,
  },
  exit: {
    type: Function,
    required: true,
  },
})
</script>

<style scoped lang="scss">
.action-groups {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: var(--gap-sm);
}

.arrow {
  transition: transform 0.2s ease-in-out;
  display: flex;
  align-items: center;
  &.rotate {
    transform: rotate(180deg);
  }
}

.status {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 0.5rem;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-button-bg);
  padding: var(--gap-sm) var(--gap-lg);
}

.running-text {
  display: flex;
  flex-direction: row;
  gap: var(--gap-xs);
  white-space: nowrap;
  overflow: hidden;
  -webkit-user-select: none; /* Safari */
  -ms-user-select: none; /* IE 10 and IE 11 */
  user-select: none;

  &.clickable:hover {
    cursor: pointer;
  }
}

.circle {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 50%;
  display: inline-block;
  margin-right: 0.25rem;

  &.running {
    background-color: var(--color-brand);
  }

  &.stopped {
    background-color: var(--color-base);
  }
}

.icon-button {
  background-color: rgba(0, 0, 0, 0);
  box-shadow: none;
  width: 1.25rem !important;
  height: 1.25rem !important;

  &.stop {
    --text-color: var(--color-red) !important;
  }
}

.info-section {
  position: absolute;
  top: 3.5rem;
  right: 0.75rem;
  z-index: 9;
  display: flex;
  flex-direction: column;
}

.info-card {
  width: 20rem;
  background-color: var(--color-raised-bg);
  box-shadow: var(--shadow-raised);
  display: flex;
  flex-direction: column;
  gap: 1rem;
  overflow: auto;
  transition: all 0.2s ease-in-out;
  border: 1px solid var(--color-button-bg);

  &.hidden {
    transform: translateY(-100%);
  }
}

.loading-option {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 0.5rem;
  margin: 0;
  padding: 0;

  :hover {
    background-color: var(--color-raised-bg-hover);
  }
}

.loading-text {
  display: flex;
  flex-direction: column;
  margin: 0;
  padding: 0;

  .row {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;
  }
}

.loading-icon {
  width: 2.25rem;
  height: 2.25rem;
  display: block;

  :deep(svg) {
    left: 1rem;
    width: 2.25rem;
    height: 2.25rem;
  }
}

.show-card-icon {
  color: var(--color-brand);
}

.download-enter-active,
.download-leave-active {
  transition: opacity 0.3s ease;
}

.download-enter-from,
.download-leave-to {
  opacity: 0;
}

.progress-bar {
  width: 100%;
}

.info-text {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.5rem;
  margin: 0;
  padding: 0;
}

.info-title {
  margin: 0;
}

.profile-button {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: var(--gap-sm);
  width: 100%;
  background-color: var(--color-raised-bg);
  box-shadow: none;

  .text {
    margin-right: auto;
  }
}

.profile-card {
  position: absolute;
  top: 3.5rem;
  right: 0.5rem;
  z-index: 9;
  background-color: var(--color-raised-bg);
  box-shadow: var(--shadow-raised);
  display: flex;
  flex-direction: column;
  overflow: auto;
  transition: all 0.2s ease-in-out;
  border: 1px solid var(--color-button-bg);
  padding: var(--gap-md);

  &.hidden {
    transform: translateY(-100%);
  }
}
</style>
