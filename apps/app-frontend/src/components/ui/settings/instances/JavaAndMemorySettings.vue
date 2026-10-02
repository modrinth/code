<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { Button, commonMessages, defineMessages, Input, Slider, useVIntl } from '@modrinth/ui'

import { useDefaultInstanceSettings } from '@/composables/use-default-instance-settings'
import useMemorySlider from '@/composables/useMemorySlider'
import { parseEnvVars, serializeEnvVars } from '@/helpers/settings'

import JavaSettings from './JavaSettings.vue'

const { formatMessage } = useVIntl()
const { settings, settingsQuery } = useDefaultInstanceSettings(
	(value) => ({
		memory: { ...value.memory },
		launchArgs: value.extra_launch_args.join(' '),
		envVars: serializeEnvVars(value.custom_env_vars),
	}),
	(value) => ({
		memory: { ...value.memory },
		extra_launch_args: value.launchArgs.trim().split(/\s+/).filter(Boolean),
		custom_env_vars: parseEnvVars(value.envVars),
	}),
)
const { maxMemory, snapPoints, memoryQuery } = useMemorySlider()

const messages = defineMessages({
	javaInstallationsTitle: {
		id: 'app.settings.tabs.java-installations',
		defaultMessage: 'Java installations',
	},
	memoryAllocationTitle: {
		id: 'app.settings.default-instance-options.memory-allocation.title',
		defaultMessage: 'Memory allocation',
	},
	memoryAllocationDescription: {
		id: 'app.settings.default-instance-options.memory-allocation.description',
		defaultMessage: 'Maximum memory available to each instance.',
	},
	javaArgumentsTitle: {
		id: 'app.settings.default-instance-options.java-arguments.title',
		defaultMessage: 'Java arguments',
	},
	javaArgumentsPlaceholder: {
		id: 'app.settings.default-instance-options.java-arguments.placeholder',
		defaultMessage: 'Enter Java arguments...',
	},
	javaArgumentsDescription: {
		id: 'app.settings.default-instance-options.java-arguments.description',
		defaultMessage: 'Arguments passed to Java when launching an instance.',
	},
	environmentVariablesTitle: {
		id: 'app.settings.default-instance-options.environment-variables.title',
		defaultMessage: 'Environment variables',
	},
	environmentVariablesPlaceholder: {
		id: 'app.settings.default-instance-options.environment-variables.placeholder',
		defaultMessage: 'Enter environment variables...',
	},
	environmentVariablesDescription: {
		id: 'app.settings.default-instance-options.environment-variables.description',
		defaultMessage: 'Environment variables set when launching an instance.',
	},
})
</script>

<template>
	<div class="flex flex-col gap-6">
		<section class="flex flex-col gap-4">
			<h2 class="m-0 text-xl font-semibold text-contrast">
				{{ formatMessage(messages.javaInstallationsTitle) }}
			</h2>
			<JavaSettings />
		</section>
		<div class="flex flex-col gap-6 border-0 border-t border-solid border-surface-5 pt-6">
			<template v-if="settings">
				<div class="flex flex-col gap-2.5">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.memoryAllocationTitle) }}
					</h2>
					<Slider
						v-if="maxMemory"
						id="max-memory"
						v-model="settings.memory.maximum"
						:aria-label="formatMessage(messages.memoryAllocationTitle)"
						:min="512"
						:max="maxMemory"
						:step="64"
						:snap-points="snapPoints"
						:snap-range="512"
						min-label="512 MB"
						:max-label="`${Number((maxMemory / 1024).toFixed(1))} GB`"
						unit="MB"
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
					<p class="m-0 mt-1 leading-tight">
						{{ formatMessage(messages.memoryAllocationDescription) }}
					</p>
				</div>
				<div class="flex flex-col gap-2.5">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.javaArgumentsTitle) }}
					</h2>
					<Input
						id="java-args"
						v-model="settings.launchArgs"
						:aria-label="formatMessage(messages.javaArgumentsTitle)"
						autocomplete="off"
						type="text"
						:placeholder="formatMessage(messages.javaArgumentsPlaceholder)"
						wrapper-class="w-full"
					/>
					<p class="m-0 leading-tight">
						{{ formatMessage(messages.javaArgumentsDescription) }}
					</p>
				</div>
				<div class="flex flex-col gap-2.5">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.environmentVariablesTitle) }}
					</h2>
					<Input
						id="env-vars"
						v-model="settings.envVars"
						:aria-label="formatMessage(messages.environmentVariablesTitle)"
						autocomplete="off"
						type="text"
						:placeholder="formatMessage(messages.environmentVariablesPlaceholder)"
						wrapper-class="w-full"
					/>
					<p class="m-0 leading-tight">
						{{ formatMessage(messages.environmentVariablesDescription) }}
					</p>
				</div>
			</template>
			<Button v-else-if="settingsQuery.isError.value" @click="settingsQuery.refetch()">
				{{ formatMessage(commonMessages.refreshButton) }}
			</Button>
			<div v-else class="flex items-center gap-2 text-secondary">
				<SpinnerIcon class="size-5 animate-spin" aria-hidden="true" />
				{{ formatMessage(commonMessages.loadingLabel) }}
			</div>
		</div>
	</div>
</template>
