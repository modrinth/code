<template>
	<div v-tooltip="'Change server loader'" class="flex min-w-0 flex-row items-center gap-2 truncate">
		<Separator v-if="!noSeparator" />
		<div class="flex flex-row items-center gap-1.5">
			<LoaderIcon v-if="loader" :loader="loader" />
			<div v-else class="size-5 shrink-0 animate-pulse rounded-full bg-button-border"></div>
			<AutoLink
				v-if="isLink"
				:to="settingsLinkTarget"
				class="flex min-w-0 items-center text-sm font-semibold"
				:class="settingsLinkTarget ? 'hover:underline' : ''"
			>
				<span v-if="loader">
					{{ loader }}
					<span v-if="loaderVersion">{{ loaderVersion }}</span>
				</span>
				<span v-else class="flex gap-2">
					<span class="inline-block h-4 w-12 animate-pulse rounded bg-button-border"></span>
					<span class="inline-block h-4 w-12 animate-pulse rounded bg-button-border"></span>
				</span>
			</AutoLink>
			<div v-else class="min-w-0 text-sm font-semibold">
				<span v-if="loader">
					{{ loader }}
					<span v-if="loaderVersion">{{ loaderVersion }}</span>
				</span>
				<span v-else class="flex gap-2">
					<span class="inline-block h-4 w-12 animate-pulse rounded bg-button-border"></span>
					<span class="inline-block h-4 w-12 animate-pulse rounded bg-button-border"></span>
				</span>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { injectServerSettingsModal } from '#ui/providers/server-settings-modal'

import AutoLink from '../../base/AutoLink.vue'
import LoaderIcon from '../icons/LoaderIcon.vue'
import Separator from './Separator.vue'

defineProps<{
	noSeparator?: boolean
	loader?: 'Fabric' | 'Quilt' | 'Forge' | 'NeoForge' | 'Paper' | 'Spigot' | 'Bukkit' | 'Vanilla'
	loaderVersion?: string
	isLink?: boolean
}>()

const settingsModal = injectServerSettingsModal(null)
const settingsLinkTarget = computed(() => {
	if (settingsModal) {
		return () => settingsModal.openServerSettings({ tabId: 'installation' })
	}
	return ''
})
</script>
