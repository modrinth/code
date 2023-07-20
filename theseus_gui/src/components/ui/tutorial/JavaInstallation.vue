<script setup>
import { Card, Button } from 'omorphia'
import {onBeforeUnmount, ref, watch} from "vue";
import {get, set} from "@/helpers/settings.js";
import {handleError} from "@/store/notifications.js";
import JavaSelector from "@/components/ui/JavaSelector.vue";
import {auto_install_java, get_jre} from "@/helpers/jre.js";
import mixpanel from "mixpanel-browser";
import {loading_listener} from "@/helpers/events.js";

async function fetchSettings() {
  const fetchSettings = await get().catch(handleError)

  if (!fetchSettings.java_globals.JAVA_17)
    fetchSettings.java_globals.JAVA_17 = { path: '', version: '' }
  else installedJava.value = true

  settings.value = fetchSettings
}
const settings = ref(null)
const autoInstalling = ref(false)
const installedJava = ref(false)
await fetchSettings()

watch([settings, settings.value], async () => {
  const setSettings = JSON.parse(JSON.stringify(settings.value))

  if (setSettings.java_globals.JAVA_17?.path === '') {
    setSettings.java_globals.JAVA_17 = undefined
  }
  if (setSettings.java_globals.JAVA_17?.path) {
    setSettings.java_globals.JAVA_17.path = setSettings.java_globals.JAVA_17.path.replace(
      'java.exe',
      'javaw.exe'
    )
    installedJava.value = true
  }

  await set(setSettings)
})

async function autoInstallJava() {
  autoInstalling.value = true
  const path = await auto_install_java(17).catch(handleError)
  if (!settings.value.java_globals) settings.value.java_globals = {}
  const version = await get_jre(path).catch(handleError)
  // weird vue bug, ignore
  settings.value.java_globals.JAVA_17 = version
  settings.value.java_globals.JAVA_17 = version
  mixpanel.track('OnboardingAutoInstallJava')
  autoInstalling.value = false
  installedJava.value = true
}

const javaLoadingEvent = ref(null)
const unlistenLoading = await loading_listener(async (event) => {
  if (event.event.type === 'java_download') {
    javaLoadingEvent.value = event
    if (!event.fraction) {
      javaLoadingEvent.value.fraction = 1
    }
  }
})

onBeforeUnmount(() => {
  unlistenLoading()
})

defineProps({
  finish: {
    type: Function,
    required: true,
  },
})
</script>

<template>
  <Card class="java-card">
    <h2>Java Installation</h2>
    <div class="markdown-body">
      <p>
        The Modrinth App requires a Java installation to run Minecraft. You can let us
        automatically install Java for you or select an existing installation below.
      </p>
    </div>
    <div class="java-section">
      <h3>Java location</h3>
      <JavaSelector v-model="settings.java_globals.JAVA_17" compact />
    </div>
    <div class="bottom-buttons">
      <Button large :disabled="autoInstalling || installedJava" @click="autoInstallJava">
        {{ installedJava ? 'Installed' : autoInstalling ? 'Installing...' : 'Install Java'}}
      </Button>
      <Button large :disabled="!installedJava" @click="finish" color="primary">Finish</Button>
    </div>
  </Card>
</template>

<style scoped lang="scss">
.java-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  width: 50rem;
}

.java-section {
  display: flex;
  flex-direction: column;
  gap: var(--gap-sm);
  padding: var(--gap-lg);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-button-bg);
}

.bottom-buttons {
  display: flex;
  flex-direction: row;
  flex-grow: 1;
  gap: var(--gap-sm);
  justify-content: center;
}
</style>
