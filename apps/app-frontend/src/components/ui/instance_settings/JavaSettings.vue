<script setup lang="ts">
import { CheckCircleIcon, XCircleIcon } from '@modrinth/assets'
import {
	Checkbox,
	defineMessages,
	injectNotificationManager,
	Slider,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { computed, readonly, ref, watch } from 'vue'

import JavaSelector from '@/components/ui/JavaSelector.vue'
import useMemorySlider from '@/composables/useMemorySlider'
import { edit, get_optimal_jre_key } from '@/helpers/profile'
import { get } from '@/helpers/settings.ts'

import type { AppSettings, InstanceSettingsTabProps } from '../../../helpers/types'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const props = defineProps<InstanceSettingsTabProps>()

const globalSettings = (await get().catch(handleError)) as unknown as AppSettings

const overrideJavaInstall = ref(!!props.instance.java_path)
const optimalJava = readonly(await get_optimal_jre_key(props.instance.path).catch(handleError))
const javaInstall = ref({ path: optimalJava.path ?? props.instance.java_path })

const overrideJavaArgs = ref((props.instance.extra_launch_args?.length ?? 0) > 0)
const javaArgs = ref(
	(props.instance.extra_launch_args ?? globalSettings.extra_launch_args).join(' '),
)

const overrideEnvVars = ref((props.instance.custom_env_vars?.length ?? 0) > 0)
const envVars = ref(
	(props.instance.custom_env_vars ?? globalSettings.custom_env_vars)
		.map((x) => x.join('='))
		.join(' '),
)

const overrideMemorySettings = ref(!!props.instance.memory)
const memory = ref(props.instance.memory ?? globalSettings.memory)
const { maxMemory, snapPoints } = (await useMemorySlider().catch(handleError)) as unknown as {
	maxMemory: number
	snapPoints: number[]
}

const editProfileObject = computed(() => {
	return {
		java_path:
			overrideJavaInstall.value && javaInstall.value.path !== ''
				? javaInstall.value.path.replace('java.exe', 'javaw.exe')
				: null,
		extra_launch_args: overrideJavaArgs.value
			? javaArgs.value.trim().split(/\s+/).filter(Boolean)
			: null,
		custom_env_vars: overrideEnvVars.value
			? envVars.value
					.trim()
					.split(/\s+/)
					.filter(Boolean)
					.map((x) => x.split('=').filter(Boolean))
			: null,
		memory: overrideMemorySettings.value ? memory.value : null,
	}
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
	customJavaInstallation: {
		id: 'instance.settings.tabs.java.custom-java-installation',
		defaultMessage: 'Custom Java installation',
	},
	usingDefaultJava: {
		id: 'instance.settings.tabs.java.using-default-java',
		defaultMessage: 'Using default Java {version} installation:',
	},
	defaultJavaNotFound: {
		id: 'instance.settings.tabs.java.default-java-not-found',
		defaultMessage: 'Could not find a default Java {version} installation. Please set one below:',
	},
	couldNotDetermineJava: {
		id: 'instance.settings.tabs.java.could-not-determine-java',
		defaultMessage:
			'Could not automatically determine a Java installation to use. Please set one below:',
	},
	javaMemory: {
		id: 'instance.settings.tabs.java.java-memory',
		defaultMessage: 'Memory allocated',
	},
	customMemoryAllocation: {
		id: 'instance.settings.tabs.java.custom-memory-allocation',
		defaultMessage: 'Custom memory allocation',
	},
	javaArguments: {
		id: 'instance.settings.tabs.java.java-arguments',
		defaultMessage: 'Java arguments',
	},
	customJavaArguments: {
		id: 'instance.settings.tabs.java.custom-java-arguments',
		defaultMessage: 'Custom java arguments',
	},
	enterJavaArguments: {
		id: 'instance.settings.tabs.java.enter-java-arguments',
		defaultMessage: 'Enter java arguments...',
	},
	javaEnvironmentVariables: {
		id: 'instance.settings.tabs.java.environment-variables',
		defaultMessage: 'Environment variables',
	},
	customEnvironmentVariables: {
		id: 'instance.settings.tabs.java.custom-environment-variables',
		defaultMessage: 'Custom environment variables',
	},
	enterEnvironmentVariables: {
		id: 'instance.settings.tabs.java.enter-environment-variables',
		defaultMessage: 'Enter environmental variables...',
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
		<Checkbox
			v-model="overrideJavaInstall"
			:label="formatMessage(messages.customJavaInstallation)"
			class="mb-2"
		/>
		<template v-if="!overrideJavaInstall">
			<div class="flex my-2 items-center gap-2 font-semibold">
				<template v-if="javaInstall">
					<CheckCircleIcon class="text-brand-green h-4 w-4" />
					<span>{{
						formatMessage(messages.usingDefaultJava, { version: optimalJava.major_version })
					}}</span>
				</template>
				<template v-else-if="optimalJava">
					<XCircleIcon class="text-brand-red h-5 w-5" />
					<span>{{
						formatMessage(messages.defaultJavaNotFound, { version: optimalJava.major_version })
					}}</span>
				</template>
				<template v-else>
					<XCircleIcon class="text-brand-red h-5 w-5" />
					<span>{{ formatMessage(messages.couldNotDetermineJava) }}</span>
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
		<Checkbox
			v-model="overrideMemorySettings"
			:label="formatMessage(messages.customMemoryAllocation)"
			class="mb-2"
		/>
		<Slider
			id="max-memory"
			v-model="memory.maximum"
			:disabled="!overrideMemorySettings"
			:min="512"
			:max="maxMemory"
			:step="64"
			:snap-points="snapPoints"
			:snap-range="512"
			unit="MB"
		/>
		<h2 id="project-name" class="mt-4 mb-1 text-lg font-extrabold text-contrast block">
			{{ formatMessage(messages.javaArguments) }}
		</h2>
		<Checkbox
			v-model="overrideJavaArgs"
			:label="formatMessage(messages.customJavaArguments)"
			class="my-2"
		/>
		<StyledInput
			id="java-args"
			v-model="javaArgs"
			autocomplete="off"
			:disabled="!overrideJavaArgs"
			:placeholder="formatMessage(messages.enterJavaArguments)"
			wrapper-class="w-full"
		/>
		<h2 id="project-name" class="mt-4 mb-1 text-lg font-extrabold text-contrast block">
			{{ formatMessage(messages.javaEnvironmentVariables) }}
		</h2>
		<Checkbox
			v-model="overrideEnvVars"
			:label="formatMessage(messages.customEnvironmentVariables)"
			class="mb-2"
		/>
		<StyledInput
			id="env-vars"
			v-model="envVars"
			autocomplete="off"
			:disabled="!overrideEnvVars"
			:placeholder="formatMessage(messages.enterEnvironmentVariables)"
			wrapper-class="w-full"
		/>
	</div>
</template>
