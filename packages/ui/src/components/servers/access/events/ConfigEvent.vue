<template>
	<BaseEvent>
		<span
			v-if="props.kind === 'properties'"
			class="inline-flex max-w-full min-w-0 flex-wrap items-center gap-1 whitespace-normal align-middle @[800px]:flex-nowrap @[800px]:whitespace-nowrap"
		>
			<span class="shrink-0">{{ formatMessage(messages.propertiesModifiedLabel) }}</span>
			<EventInlineText
				:text="propertiesLabel"
				class="align-middle font-mono text-[0.925em] font-medium text-contrast"
			/>
		</span>
		<IntlFormatted v-else :message-id="message">
			<template #version>
				<EventInlineText :text="newVersion ?? ''" class="align-middle font-medium text-contrast" />
			</template>
			<template #loader>
				<EventInlineText :text="newLoaderLabel" class="align-middle font-medium text-contrast" />
			</template>
			<template #command>
				<EventInlineText
					:text="command ?? ''"
					class="align-middle font-mono font-medium text-contrast"
				/>
			</template>
			<template #vendor>
				<EventInlineText :text="vendor ?? ''" class="align-middle font-medium text-contrast" />
			</template>
			<template #java-version>
				<EventInlineText :text="version ?? ''" class="align-middle font-medium text-contrast" />
			</template>
		</IntlFormatted>
	</BaseEvent>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { defineMessages, type MessageDescriptor, useVIntl } from '../../../../composables/i18n'
import IntlFormatted from '../../../base/IntlFormatted.vue'
import BaseEvent from './BaseEvent.vue'
import EventInlineText from './EventInlineText.vue'
import type { EventEntity } from './types'

const props = defineProps<{
	kind:
		| 'loader_version'
		| 'game_version'
		| 'properties'
		| 'startup_command'
		| 'java_runtime'
		| 'java_version'
	newVersion?: string | null
	newLoader?: string | null
	properties?: EventEntity[]
	command?: string
	vendor?: string
	version?: number
}>()

const messages = defineMessages({
	loaderVersionChanged: {
		id: 'servers.audit-log.event.loader-version-changed',
		defaultMessage: 'Changed loader version to <version></version>',
	},
	loaderChanged: {
		id: 'servers.audit-log.event.loader-changed',
		defaultMessage: 'Changed loader to <loader></loader>',
	},
	loaderAndVersionChanged: {
		id: 'servers.audit-log.event.loader-and-version-changed',
		defaultMessage: 'Changed loader to <loader></loader> <version></version>',
	},
	loaderVersionCleared: {
		id: 'servers.audit-log.event.loader-version-cleared',
		defaultMessage: 'Cleared loader version',
	},
	gameVersionChanged: {
		id: 'servers.audit-log.event.game-version-changed',
		defaultMessage: 'Changed Minecraft version to <version></version>',
	},
	propertiesModified: {
		id: 'servers.audit-log.event.server-properties-modified',
		defaultMessage: 'Modified server properties <properties></properties>',
	},
	propertiesModifiedLabel: {
		id: 'servers.audit-log.event.server-properties-modified-label',
		defaultMessage: 'Modified server properties',
	},
	startupCommandModified: {
		id: 'servers.audit-log.event.startup-command-modified',
		defaultMessage: 'Changed startup command to <command></command>',
	},
	javaRuntimeModified: {
		id: 'servers.audit-log.event.java-runtime-modified',
		defaultMessage: 'Changed Java runtime to <vendor></vendor>',
	},
	javaVersionModified: {
		id: 'servers.audit-log.event.java-version-modified',
		defaultMessage: 'Changed Java version to <java-version></java-version>',
	},
	configChanged: {
		id: 'servers.audit-log.event.config-changed',
		defaultMessage: 'Changed server configuration',
	},
})

const { formatMessage } = useVIntl()
const propertiesLabel = computed(
	() => props.properties?.map((property) => property.label).join(', ') ?? '',
)
const newLoader = computed(() =>
	props.kind === 'loader_version' && props.newLoader == null ? 'vanilla' : props.newLoader,
)
const newLoaderLabel = computed(() => formatLoader(newLoader.value))

const kindMessages: Record<string, MessageDescriptor> = {
	game_version: messages.gameVersionChanged,
	properties: messages.propertiesModified,
	startup_command: messages.startupCommandModified,
	java_runtime: messages.javaRuntimeModified,
	java_version: messages.javaVersionModified,
}

const message = computed(() => {
	if (props.kind === 'loader_version') {
		if (newLoader.value && props.newVersion) return messages.loaderAndVersionChanged
		if (newLoader.value) return messages.loaderChanged
		return props.newVersion == null ? messages.loaderVersionCleared : messages.loaderVersionChanged
	}
	return kindMessages[props.kind] ?? messages.configChanged
})

function formatLoader(loader: string | null | undefined): string {
	if (!loader) return ''
	return loader
		.split(/[-_]/)
		.filter(Boolean)
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ')
}
</script>
