<template>
  <div v-if="currentProcesses[0]" class="status">
    <span class="circle running" />
    <span class="running-text">
      {{ currentProcesses[0].metadata.name }}
    </span>
    <Button icon-only class="icon-button stop" @click="stop()">
      <StopIcon />
    </Button>
    <Button icon-only class="icon-button" @click="goToTerminal()">
      <TerminalIcon />
    </Button>
  </div>
  <div v-else class="status">
    <span class="circle stopped" />
    <span class="running-text"> No running profiles </span>
  </div>
</template>

<script setup>
import { Button } from 'omorphia'
import { StopIcon, TerminalIcon } from '@/assets/icons'
import { ref } from 'vue'
import {
  get_all_running_profiles as getRunningProfiles,
  kill_by_uuid as killProfile,
  get_uuids_by_profile_path as getProfileProcesses,
} from '@/helpers/process'
import { process_listener } from '@/helpers/events'
import { useRouter } from 'vue-router'

const router = useRouter()

const currentProcesses = ref(await getRunningProfiles())

await process_listener(async () => {
  await refresh()
})

const refresh = async () => {
  currentProcesses.value = await getRunningProfiles()
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
</style>
