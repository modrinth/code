<script setup lang="ts">
import {
	Button,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	Input,
	Slider,
	useVIntl,
} from '@modrinth/ui'

import JavaSelector from '@/components/ui/JavaSelector.vue'
import useMemorySlider from '@/composables/useMemorySlider'
import { edit, get_recommended_java_version } from '@/helpers/instance'
import { get_java_versions } from '@/helpers/jre'
import { get, parseEnvVars, serializeEnvVars } from '@/helpers/settings.ts'
import type { AppSettings } from '@/helpers/types'

import DefaultSettingsIntro from './default-settings-intro.vue'
import { injectInstanceSettings } from './instance-settings-context'
import ResetToDefaultButton from './reset-to-default-button.vue'
import { useInstanceSettingsDraft } from './use-instance-settings-draft'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const { instance } = injectInstanceSettings()
const { createDraft } = useInstanceSettingsDraft()
const { maxMemory, snapPoints, memoryQuery } = useMemorySlider()

const globalSettings = (await get().catch(handleError)) as unknown as AppSettings

const [recommendedJavaVersion, javaVersions] = await Promise.all([
	get_recommended_java_version(instance.value.id).catch(handleError),
	get_java_versions().catch(handleError),
])

const defaultJavaPath: string =
	(recommendedJavaVersion ? javaVersions?.[recommendedJavaVersion]?.path : undefined) ?? ''
const defaultJavaArgs = globalSettings.extra_launch_args.join(' ')
const defaultEnvVars = serializeEnvVars(globalSettings.custom_env_vars)
const defaultMemory = globalSettings.memory.maximum

const draft = createDraft<{
	javaPath: string | null
	javaArgs: string | null
	envVars: string | null
	memory: number | null
}>(
	{
		javaPath: instance.value.java_path ?? null,
		javaArgs: instance.value.extra_launch_args?.join(' ') ?? null,
		envVars: instance.value.custom_env_vars
			? serializeEnvVars(instance.value.custom_env_vars)
			: null,
		memory: instance.value.memory?.maximum ?? null,
	},
	(value) =>
		edit(instance.value.id, {
			java_path: value.javaPath ? value.javaPath.replace('java.exe', 'javaw.exe') : null,
			extra_launch_args:
				value.javaArgs !== null ? value.javaArgs.trim().split(/\s+/).filter(Boolean) : null,
			custom_env_vars: value.envVars !== null ? parseEnvVars(value.envVars) : null,
			memory: value.memory !== null ? { maximum: value.memory } : null,
		}),
)

const messages = defineMessages({
	defaultSettingsDescription: {
		id: 'instance.settings.tabs.java.default-settings-description',
		defaultMessage:
			'Your instance uses the default Java and memory settings unless you override them here.',
	},
	javaInstallation: {
		id: 'instance.settings.tabs.java.java-installation',
		defaultMessage: 'Java installation',
	},
	recommendedJavaVersion: {
		id: 'instance.settings.tabs.java.recommended-java-version',
		defaultMessage: 'Java {version} is recommended for the Minecraft version of this instance.',
	},
	javaPathPlaceholder: {
		id: 'instance.settings.tabs.java.java-path-placeholder',
		defaultMessage: '/path/to/java',
	},
	javaMemory: {
		id: 'instance.settings.tabs.java.java-memory',
		defaultMessage: 'Memory allocation',
	},
	javaArguments: {
		id: 'instance.settings.tabs.java.java-arguments',
		defaultMessage: 'Java arguments',
	},
	enterJavaArguments: {
		id: 'instance.settings.tabs.java.enter-java-arguments',
		defaultMessage: 'Enter Java arguments...',
	},
	javaEnvironmentVariables: {
		id: 'instance.settings.tabs.java.environment-variables',
		defaultMessage: 'Environment variables',
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
	<div class="flex flex-col gap-6">
		<DefaultSettingsIntro
			tab="java"
			:description="formatMessage(messages.defaultSettingsDescription)"
		/>

		<section class="flex flex-col">
			<div class="flex items-center justify-between gap-4">
				<h2 class="m-0 min-w-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.javaMemory) }}
				</h2>
				<ResetToDefaultButton
					:disabled="draft.memory === null"
					:setting="formatMessage(messages.javaMemory)"
					@reset="draft.memory = null"
				/>
			</div>
			<div class="pt-3">
				<Slider
					v-if="maxMemory"
					id="max-memory"
					:model-value="draft.memory ?? defaultMemory"
					:min="512"
					:max="maxMemory"
					:step="64"
					:snap-points="snapPoints"
					:snap-range="512"
					min-label="512 MB"
					:max-label="`${Number((maxMemory / 1024).toFixed(1))} GB`"
					unit="MB"
					@update:model-value="(value) => (draft.memory = value)"
				/>
				<Button v-else-if="memoryQuery.isError.value" @click="memoryQuery.refetch()">
					{{ formatMessage(commonMessages.refreshButton) }}
				</Button>
				<div
					v-else
					role="status"
					:aria-label="formatMessage(commonMessages.loadingLabel)"
					class="h-10 animate-pulse rounded-lg bg-surface-3"
				/>
			</div>
		</section>

		<section class="flex flex-col">
			<div class="flex items-center justify-between gap-4">
				<div class="flex min-w-0 flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.javaInstallation) }}
					</h2>
					<p v-if="recommendedJavaVersion" class="m-0 text-secondary">
						{{
							formatMessage(messages.recommendedJavaVersion, { version: recommendedJavaVersion })
						}}
					</p>
				</div>
				<ResetToDefaultButton
					:disabled="draft.javaPath === null"
					:setting="formatMessage(messages.javaInstallation)"
					@reset="draft.javaPath = null"
				/>
			</div>
			<div class="pt-3">
				<JavaSelector
					id="java-installation"
					:model-value="{ path: draft.javaPath ?? defaultJavaPath }"
					:placeholder="formatMessage(messages.javaPathPlaceholder)"
					@update:model-value="(val) => (draft.javaPath = val?.path ?? '')"
				/>
			</div>
		</section>

		<section class="flex flex-col">
			<div class="flex items-center justify-between gap-4">
				<h2 class="m-0 min-w-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.javaArguments) }}
				</h2>
				<ResetToDefaultButton
					:disabled="draft.javaArgs === null"
					:setting="formatMessage(messages.javaArguments)"
					@reset="draft.javaArgs = null"
				/>
			</div>
			<div class="pt-3">
				<Input
					id="java-args"
					:model-value="draft.javaArgs ?? defaultJavaArgs"
					autocomplete="off"
					:placeholder="formatMessage(messages.enterJavaArguments)"
					wrapper-class="w-full"
					@update:model-value="(value) => (draft.javaArgs = String(value ?? ''))"
				/>
			</div>
		</section>

		<section class="flex flex-col">
			<div class="flex items-center justify-between gap-4">
				<h2 class="m-0 min-w-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.javaEnvironmentVariables) }}
				</h2>
				<ResetToDefaultButton
					:disabled="draft.envVars === null"
					:setting="formatMessage(messages.javaEnvironmentVariables)"
					@reset="draft.envVars = null"
				/>
			</div>
			<div class="pt-3">
				<Input
					id="env-vars"
					:model-value="draft.envVars ?? defaultEnvVars"
					autocomplete="off"
					:placeholder="formatMessage(messages.enterEnvironmentVariables)"
					wrapper-class="w-full"
					@update:model-value="(value) => (draft.envVars = String(value ?? ''))"
				/>
			</div>
		</section>
	</div>
</template>
