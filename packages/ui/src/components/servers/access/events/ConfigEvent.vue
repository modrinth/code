<template>
	<BaseEvent>
		<IntlFormatted :message-id="message">
			<template #version>
				<span class="font-semibold text-contrast">{{ newVersion }}</span>
			</template>
			<template #properties>
				<EventEntityList :entities="properties ?? []" />
			</template>
			<template #command>
				<span
					v-tooltip="command"
					class="inline-block max-w-full truncate align-bottom font-mono text-contrast"
				>
					{{ command }}
				</span>
			</template>
			<template #vendor>
				<span class="font-semibold text-contrast">{{ vendor }}</span>
			</template>
			<template #java-version>
				<span class="font-semibold text-contrast">{{ version }}</span>
			</template>
		</IntlFormatted>
	</BaseEvent>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { defineMessages, type MessageDescriptor } from '../../../../composables/i18n'
import IntlFormatted from '../../../base/IntlFormatted.vue'
import BaseEvent from './BaseEvent.vue'
import EventEntityList from './EventEntityList.vue'
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

const kindMessages: Record<string, MessageDescriptor> = {
	game_version: messages.gameVersionChanged,
	properties: messages.propertiesModified,
	startup_command: messages.startupCommandModified,
	java_runtime: messages.javaRuntimeModified,
	java_version: messages.javaVersionModified,
}

const message = computed(() => {
	if (props.kind === 'loader_version') {
		return props.newVersion == null ? messages.loaderVersionCleared : messages.loaderVersionChanged
	}
	return kindMessages[props.kind] ?? messages.configChanged
})
</script>
