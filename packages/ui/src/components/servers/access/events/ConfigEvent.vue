<template>
	<BaseEvent>
		<template v-if="kind === 'loader_version'">
			Changed loader version
			<span class="font-semibold text-contrast">{{ newVersion ?? 'cleared' }}</span>
		</template>
		<template v-else-if="kind === 'game_version'">
			Changed Minecraft version to <span class="font-semibold text-contrast">{{ newVersion }}</span>
		</template>
		<template v-else-if="kind === 'properties'">
			Modified server properties <EventEntityList :entities="properties ?? []" />
		</template>
		<template v-else-if="kind === 'startup_command'">
			Changed startup command
			<span
				v-tooltip="command"
				class="inline-block max-w-full truncate align-bottom font-mono text-contrast"
			>
				{{ command }}
			</span>
		</template>
		<template v-else-if="kind === 'java_runtime'">
			Changed Java runtime to <span class="font-semibold text-contrast">{{ vendor }}</span>
		</template>
		<template v-else>
			Changed Java version to <span class="font-semibold text-contrast">{{ version }}</span>
		</template>
	</BaseEvent>
</template>

<script setup lang="ts">
import BaseEvent from './BaseEvent.vue'
import EventEntityList from './EventEntityList.vue'
import type { EventEntity } from './types'

defineProps<{
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
</script>
