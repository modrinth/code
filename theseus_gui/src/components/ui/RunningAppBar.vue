<template>
  <div v-if="currentProcesses[0]" class="status">
    <span class="circle running" />
    <span class="running-text">
      {{ currentProcesses[0].metadata.name }}
    </span>
    <Button icon-only class="icon-button stop" @click="stop()">
      <StopCircleIcon />
    </Button>
    <Button icon-only class="icon-button" @click="goToTerminal()">
      <TerminalSquareIcon />
    </Button>
    <Button
      v-if="currentLoadingBars.length > 0"
      ref="infoButton"
      icon-only
      class="icon-button show-card-icon"
      @click="toggleCard()"
    >
      <DownloadIcon />
    </Button>
  </div>
  <div v-else class="status">
    <span class="circle stopped" />
    <span class="running-text"> No running profiles </span>
    <Button
      v-if="currentLoadingBars.length > 0"
      ref="infoButton"
      icon-only
      class="icon-button show-card-icon"
      @click="toggleCard()"
    >
      <DownloadIcon />
    </Button>
  </div>
  <transition name="download">
    <Card v-if="showCard === true" ref="card" class="info-card">
      <div v-for="loadingBar in currentLoadingBars" :key="loadingBar.id" class="info-text">
        <h3 class="info-title">
          {{ loadingBar.bar_type.pack_name ?? 'Installing Modpack' }}
        </h3>
        <ProgressBar :progress="Math.floor(loadingBar.current)" />
        <div class="row">{{ Math.floor(loadingBar.current) }}% {{ loadingBar.message }}</div>
      </div>
    </Card>
  </transition>
</template>

<script setup>
import { Button, DownloadIcon, Card, StopCircleIcon, TerminalSquareIcon } from 'omorphia'
import { onBeforeUnmount, onMounted, ref } from 'vue'
import {
  get_all_running_profiles as getRunningProfiles,
  kill_by_uuid as killProfile,
  get_uuids_by_profile_path as getProfileProcesses,
} from '@/helpers/process'
import { loading_listener, process_listener } from '@/helpers/events'
import { useRouter } from 'vue-router'
import { progress_bars_list } from '@/helpers/state.js'
import ProgressBar from '@/components/ui/ProgressBar.vue'
import { handleError } from '@/store/notifications.js'

const router = useRouter()
const card = ref(null)
const infoButton = ref(null)
const showCard = ref(false)

const currentProcesses = ref(await getRunningProfiles().catch(handleError))

await process_listener(async () => {
  await refresh()
})

const refresh = async () => {
  currentProcesses.value = await getRunningProfiles().catch(handleError)
}

const stop = async () => {
  try {
    const processes = await getProfileProcesses(currentProcesses.value[0].path)
    await killProfile(processes[0])
  } catch (e) {
    console.error(e)
  }
  await refresh()
}

const goToTerminal = () => {
  router.push(`/instance/${encodeURIComponent(currentProcesses.value[0].path)}/logs`)
}

const currentLoadingBars = ref(Object.values(await progress_bars_list().catch(handleError)))

await loading_listener(async () => {
  await refreshInfo()
})

const refreshInfo = async () => {
  const currentLoadingBarCount = currentLoadingBars.value.length
  currentLoadingBars.value = Object.values(await progress_bars_list().catch(handleError))
  if (currentLoadingBars.value.length === 0) {
    showCard.value = false
  } else if (currentLoadingBarCount < currentLoadingBars.value.length) {
    showCard.value = true
  }
}

const handleClickOutside = (event) => {
  if (
    card.value &&
    infoButton.value.$el !== event.target &&
    card.value.$el !== event.target &&
    !document.elementsFromPoint(event.clientX, event.clientY).includes(card.value.$el) &&
    !document.elementsFromPoint(event.clientX, event.clientY).includes(infoButton.value.$el)
  ) {
    showCard.value = false
  }
}

const toggleCard = async () => {
  showCard.value = !showCard.value
  await refreshInfo()
}

onMounted(() => {
  window.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  window.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped lang="scss">
.status {
  height: 100%;
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 0.5rem;
  background-color: var(--color-raised-bg);
  padding: 0 1rem;
  margin: 0;
}

.running-text {
  white-space: nowrap;
  overflow: hidden;
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

.info-card {
  position: absolute;
  top: 3.5rem;
  right: 0.5rem;
  z-index: 9;
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
</style>
