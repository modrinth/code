<script setup lang="ts">
import { ExternalIcon } from '@modrinth/assets'
import { defineMessages, IntlFormatted, SegmentedControl, useVIntl } from '@modrinth/ui'
import {
	injectSettingsActions,
	type InstanceDefaultsTab,
} from '@/features/settings/settings-actions'

import { injectInstanceSettings } from './instance-settings-context'
import type { SettingsSource } from './use-settings-source'

const props = defineProps<{ title: string; tab: InstanceDefaultsTab }>()
const source = defineModel<SettingsSource>({ required: true })

const { formatMessage } = useVIntl()
const { closeModal } = injectInstanceSettings()
const { openInstanceDefaults } = injectSettingsActions()

const messages = defineMessages({
	usingDefaults: {
		id: 'instance.settings.source.using-defaults',
		defaultMessage: 'Using defaults from <link>app settings</link>',
	},
	usingCustom: {
		id: 'instance.settings.source.using-custom',
		defaultMessage: 'Using custom settings for this instance',
	},
	appSettings: {
		id: 'instance.settings.source.app-settings',
		defaultMessage: 'App settings',
	},
	custom: {
		id: 'instance.settings.source.custom',
		defaultMessage: 'Custom',
	},
})

const sources: SettingsSource[] = ['app', 'custom']
const neutralSources: SettingsSource[] = ['app']

function formatSource(value: unknown): string {
	return formatMessage(value === 'app' ? messages.appSettings : messages.custom)
}

function setSource(value: unknown) {
	if (value === 'app' || value === 'custom') source.value = value
}

function showDefaultSettings() {
	const openSettings = () => openInstanceDefaults(props.tab)
	if (closeModal) {
		closeModal(openSettings)
	} else {
		openSettings()
	}
}
</script>

<template>
	<div
		class="flex items-center justify-between gap-4 border-0 border-b border-solid border-surface-5 pb-6"
	>
		<div class="flex min-w-0 flex-col gap-1">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ title }}
			</h2>
			<p class="m-0 text-secondary">
				<IntlFormatted v-if="source === 'app'" :message-id="messages.usingDefaults">
					<template #link="{ children }">
						<button
							type="button"
							class="inline-flex cursor-pointer items-center gap-1 border-0 bg-transparent p-0 align-bottom text-[length:inherit] font-medium text-blue hover:underline"
							@click="showDefaultSettings"
						>
							<component :is="() => children" />
							<ExternalIcon class="size-4" aria-hidden="true" />
						</button>
					</template>
				</IntlFormatted>
				<template v-else>{{ formatMessage(messages.usingCustom) }}</template>
			</p>
		</div>
		<SegmentedControl
			:model-value="source"
			class="shrink-0"
			:items="sources"
			:format-label="formatSource"
			:neutral-items="neutralSources"
			v-bind="{ ariaLabel: title }"
			@update:model-value="setSource"
		/>
	</div>
</template>
