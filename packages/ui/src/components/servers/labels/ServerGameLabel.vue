<template>
	<div v-if="game" class="min-w-0 flex-none flex-row items-center gap-1.5 first:!flex">
		<Separator v-if="!noSeparator" />

		<GameIcon aria-hidden="true" />
		<AutoLink
			v-if="isLink"
			:to="settingsLinkTarget"
			class="flex min-w-0 items-center truncate text-sm font-medium"
			:class="settingsLinkTarget ? 'hover:underline' : ''"
		>
			<div class="flex flex-row items-center gap-1">
				{{ game[0].toUpperCase() + game.slice(1) }}
				<span v-if="mcVersion">{{ mcVersion }}</span>
				<span v-else class="inline-block h-3 w-12 animate-pulse rounded bg-button-border"></span>
			</div>
		</AutoLink>
		<div
			v-else
			v-tooltip="'Change server version'"
			class="pointer-events-none flex min-w-0 flex-row items-center gap-1 truncate text-sm font-medium"
		>
			{{ game[0].toUpperCase() + game.slice(1) }}
			<span v-if="mcVersion">{{ mcVersion }}</span>
			<span v-else class="inline-block h-3 w-16 animate-pulse rounded bg-button-border"></span>
		</div>
	</div>
</template>

<script setup lang="ts">
import { GameIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { injectServerSettingsModal } from '#ui/providers/server-settings-modal'

import AutoLink from '../../base/AutoLink.vue'
import Separator from './Separator.vue'

defineProps<{
	game: string
	mcVersion: string
	isLink?: boolean
	noSeparator?: boolean
}>()

const settingsModal = injectServerSettingsModal(null)
const settingsLinkTarget = computed(() => {
	if (settingsModal) {
		return () => settingsModal.openServerSettings({ tabId: 'installation' })
	}
	return ''
})
</script>
