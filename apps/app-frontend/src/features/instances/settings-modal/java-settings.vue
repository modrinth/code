<script setup lang="ts">
import { defineMessages, injectNotificationManager, Input, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import JavaSelector from '@/features/java/java-selector/index.vue'
import MemorySlider from '@/features/java/memory-slider.vue'
import { javaVersionsQueryOptions } from '@/features/java/queries'
import { edit } from '@/helpers/instance'
import { appSettingsQueryOptions, parseEnvVars, serializeEnvVars } from '@/helpers/settings.ts'
import { instanceRecommendedJavaQueryOptions } from '@/pages/instance/query-options'

import DefaultSettingsIntro from './default-settings-intro.vue'
import { injectInstanceSettings } from './instance-settings-context'
import ResetToDefaultButton from './reset-to-default-button.vue'
import { useInstanceSettingsDraft } from './use-instance-settings-draft'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const { instance } = injectInstanceSettings()

const draft = useInstanceSettingsDraft(
	() => ({
		javaPath: instance.value.java_path ?? null,
		javaArgs: instance.value.extra_launch_args?.join(' ') ?? null,
		envVars: instance.value.custom_env_vars
			? serializeEnvVars(instance.value.custom_env_vars)
			: null,
		memory: instance.value.memory?.maximum ?? null,
	}),
	(value) =>
		edit(instance.value.id, {
			java_path: value.javaPath ? value.javaPath.replace('java.exe', 'javaw.exe') : null,
			extra_launch_args:
				value.javaArgs !== null ? value.javaArgs.trim().split(/\s+/).filter(Boolean) : null,
			custom_env_vars: value.envVars !== null ? parseEnvVars(value.envVars) : null,
			memory: value.memory !== null ? { maximum: value.memory } : null,
		}),
)

const settingsQuery = useQuery(appSettingsQueryOptions())
const javaVersionsQuery = useQuery(javaVersionsQueryOptions())
const recommendedJavaQuery = useQuery(
	computed(() => instanceRecommendedJavaQueryOptions(instance.value.id)),
)

await Promise.all([
	settingsQuery.suspense(),
	javaVersionsQuery.suspense(),
	recommendedJavaQuery.suspense(),
]).catch(handleError)

const recommendedJavaVersion = computed(() => recommendedJavaQuery.data.value)
const defaultJavaPath = computed(() =>
	recommendedJavaVersion.value
		? (javaVersionsQuery.data.value?.[recommendedJavaVersion.value]?.path ?? '')
		: '',
)
const defaultJavaArgs = computed(() => settingsQuery.data.value?.extra_launch_args.join(' ') ?? '')
const defaultEnvVars = computed(() => serializeEnvVars(settingsQuery.data.value?.custom_env_vars))
const defaultMemory = computed(() => settingsQuery.data.value?.memory.maximum ?? 0)

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
				<MemorySlider
					id="max-memory"
					:model-value="draft.memory ?? defaultMemory"
					:aria-label="formatMessage(messages.javaMemory)"
					@update:model-value="(value) => (draft.memory = value)"
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
