<script setup lang="ts">
import { Checkbox, Slider } from '@modrinth/ui'
import { CheckCircleIcon, XCircleIcon } from '@modrinth/assets'
import { computed, readonly, ref, watch } from 'vue'
import { edit, get_optimal_jre_key } from '@/helpers/profile'
import { handleError } from '@/store/notifications'
import { defineMessages, useVIntl } from '@vintl/vintl'
import JavaSelector from '@/components/ui/JavaSelector.vue'
import { get_max_memory } from '@/helpers/jre'
import { get } from '@/helpers/settings.ts'
import type { InstanceSettingsTabProps, AppSettings, MemorySettings } from '../../../helpers/types'

const { formatMessage } = useVIntl()

const props = defineProps<InstanceSettingsTabProps>()

const globalSettings = (await get().catch(handleError)) as AppSettings

const overrideJavaInstall = ref(!!props.instance.java_path)
const optimalJava = readonly(await get_optimal_jre_key(props.instance.path).catch(handleError))
const javaInstall = ref({ path: optimalJava.path ?? props.instance.java_path })

const overrideJavaArgs = ref(props.instance.extra_launch_args?.length !== undefined)
const javaArgs = ref(
  (props.instance.extra_launch_args ?? globalSettings.extra_launch_args).join(' '),
)

const overrideEnvVars = ref(props.instance.custom_env_vars?.length !== undefined)
const envVars = ref(
  (props.instance.custom_env_vars ?? globalSettings.custom_env_vars)
    .map((x) => x.join('='))
    .join(' '),
)

const overrideMemorySettings = ref(!!props.instance.memory)
const memory = ref(props.instance.memory ?? globalSettings.memory)
const maxMemory = Math.floor((await get_max_memory().catch(handleError)) / 1024)

const editProfileObject = computed(() => {
  const editProfile: {
    java_path?: string
    extra_launch_args?: string[]
    custom_env_vars?: string[][]
    memory?: MemorySettings
  } = {}

  if (overrideJavaInstall.value) {
    if (javaInstall.value.path !== '') {
      editProfile.java_path = javaInstall.value.path.replace('java.exe', 'javaw.exe')
    }
  }

  if (overrideJavaArgs.value) {
    editProfile.extra_launch_args = javaArgs.value.trim().split(/\s+/).filter(Boolean)
  }

  if (overrideEnvVars.value) {
    editProfile.custom_env_vars = envVars.value
      .trim()
      .split(/\s+/)
      .filter(Boolean)
      .map((x) => x.split('=').filter(Boolean))
  }

  if (overrideMemorySettings.value) {
    editProfile.memory = memory.value
  }

  return editProfile
})

watch(
  [
    overrideJavaInstall,
    javaInstall,
    overrideJavaArgs,
    javaArgs,
    overrideEnvVars,
    envVars,
    overrideMemorySettings,
    memory,
  ],
  async () => {
    await edit(props.instance.path, editProfileObject.value)
  },
  { deep: true },
)

const messages = defineMessages({
  javaInstallation: {
    id: 'instance.settings.tabs.java.java-installation',
    defaultMessage: 'Java installation',
  },
  javaArguments: {
    id: 'instance.settings.tabs.java.java-arguments',
    defaultMessage: 'Java arguments',
  },
  javaEnvironmentVariables: {
    id: 'instance.settings.tabs.java.environment-variables',
    defaultMessage: 'Environment variables',
  },
  javaMemory: {
    id: 'instance.settings.tabs.java.java-memory',
    defaultMessage: 'Memory allocated',
  },
  hooks: {
    id: 'instance.settings.tabs.java.hooks',
    defaultMessage: 'Hooks',
  },
})
</script>

<template>
  <div>
    <h2 id="project-name" class="m-0 mb-1 text-lg font-extrabold text-contrast block">
      {{ formatMessage(messages.javaInstallation) }}
    </h2>
    <Checkbox v-model="overrideJavaInstall" label="Custom Java installation" class="mb-2" />
    <template v-if="!overrideJavaInstall">
      <div class="flex my-2 items-center gap-2 font-semibold">
        <template v-if="javaInstall">
          <CheckCircleIcon class="text-brand-green h-4 w-4" />
          <span>Using default Java {{ optimalJava.major_version }} installation:</span>
        </template>
        <template v-else-if="optimalJava">
          <XCircleIcon class="text-brand-red h-5 w-5" />
          <span
            >Could not find a default Java {{ optimalJava.major_version }} installation. Please set
            one below:</span
          >
        </template>
        <template v-else>
          <XCircleIcon class="text-brand-red h-5 w-5" />
          <span
            >Could not automatically determine a Java installation to use. Please set one
            below:</span
          >
        </template>
      </div>
      <div
        v-if="javaInstall && !overrideJavaInstall"
        class="p-4 bg-bg rounded-xl text-xs text-secondary leading-none font-mono"
      >
        {{ javaInstall.path }}
      </div>
    </template>
    <JavaSelector v-if="overrideJavaInstall || !javaInstall" v-model="javaInstall" />
    <h2 id="project-name" class="mt-4 mb-1 text-lg font-extrabold text-contrast block">
      {{ formatMessage(messages.javaMemory) }}
    </h2>
    <Checkbox v-model="overrideMemorySettings" label="Custom memory allocation" class="mb-2" />
    <Slider
      id="max-memory"
      v-model="memory.maximum"
      :disabled="!overrideMemorySettings"
      :min="512"
      :max="maxMemory"
      :step="64"
      unit="MB"
    />
    <h2 id="project-name" class="mt-4 mb-1 text-lg font-extrabold text-contrast block">
      {{ formatMessage(messages.javaArguments) }}
    </h2>
    <Checkbox v-model="overrideJavaArgs" label="Custom java arguments" class="my-2" />
    <input
      id="java-args"
      v-model="javaArgs"
      autocomplete="off"
      :disabled="!overrideJavaArgs"
      type="text"
      class="w-full"
      placeholder="Enter java arguments..."
    />
    <h2 id="project-name" class="mt-4 mb-1 text-lg font-extrabold text-contrast block">
      {{ formatMessage(messages.javaEnvironmentVariables) }}
    </h2>
    <Checkbox v-model="overrideEnvVars" label="Custom environment variables" class="mb-2" />
    <input
      id="env-vars"
      v-model="envVars"
      autocomplete="off"
      :disabled="!overrideEnvVars"
      type="text"
      class="w-full"
      placeholder="Enter environmental variables..."
    />
  </div>
</template>
