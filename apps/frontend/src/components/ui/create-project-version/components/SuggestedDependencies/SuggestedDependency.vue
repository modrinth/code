<template>
	<div
		class="flex items-center justify-between gap-2 rounded-xl bg-button-bg px-4 py-1 text-button-text"
	>
		<div class="grid max-w-[75%] grid-cols-[auto_1fr_auto] items-center gap-2">
			<Avatar v-if="icon" :src="icon" alt="dependency-icon" size="20px" :no-shadow="true" />

			<span v-tooltip="name || 'Unknown Project'" class="truncate font-semibold text-contrast">
				{{ name || 'Unknown Project' }}
			</span>

			<TagItem class="shrink-0 border !border-solid border-surface-5">
				{{ dependencyType }}
			</TagItem>
		</div>

		<span
			v-if="versionName"
			v-tooltip="versionName"
			class="max-w-[35%] truncate whitespace-nowrap font-medium"
		>
			{{ versionName }}
		</span>

		<div class="flex items-center justify-end gap-1">
			<ButtonStyled size="standard" :circular="true">
				<button aria-label="Add dependency" class="!shadow-none" @click="emitAddSuggestion">
					<PlusIcon aria-hidden="true" />
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { PlusIcon } from '@modrinth/assets'
import { Avatar, ButtonStyled, TagItem } from '@modrinth/ui'

const emit = defineEmits<{
	(e: 'onAddSuggestion'): void
}>()

const { name, icon, dependencyType, versionName } = defineProps<{
	name?: string
	icon?: string
	dependencyType: Labrinth.Versions.v2.DependencyType
	versionName?: string
}>()

function emitAddSuggestion() {
	emit('onAddSuggestion')
}
</script>
