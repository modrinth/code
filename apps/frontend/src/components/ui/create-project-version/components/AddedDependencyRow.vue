<template>
	<div
		class="flex h-11 items-center justify-between gap-2 rounded-xl bg-button-bg px-4 py-1 text-button-text"
	>
		<div class="grid max-w-[75%] grid-cols-[auto_1fr_auto] items-center gap-2">
			<Avatar v-if="icon" :src="icon" alt="dependency-icon" size="20px" :no-shadow="true" />

			<span v-tooltip="name || projectId" class="truncate font-semibold text-contrast">
				{{ name || 'Unknown Project' }}
			</span>

			<TagItem class="shrink-0 border !border-solid border-surface-5 capitalize">
				{{ dependencyType }}
			</TagItem>
		</div>

		<span
			v-if="versionName"
			v-tooltip="versionName"
			class="truncate whitespace-nowrap font-medium"
			:class="!hideRemove ? 'max-w-[35%]' : 'max-w-[50%]'"
		>
			{{ versionName }}
		</span>

		<div v-if="!hideRemove" class="flex items-center justify-end gap-1">
			<ButtonStyled size="standard" :circular="true">
				<button aria-label="Remove file" class="-mr-2 !shadow-none" @click="emitRemove">
					<XIcon aria-hidden="true" />
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { XIcon } from '@modrinth/assets'
import { Avatar, ButtonStyled, TagItem } from '@modrinth/ui'

const emit = defineEmits<{
	(e: 'fileTypeChange', type: string): void
	(e: 'remove'): void
}>()

const { projectId, name, icon, dependencyType, versionName, hideRemove } = defineProps<{
	projectId: string
	name?: string
	icon?: string
	dependencyType: Labrinth.Versions.v2.DependencyType
	versionName?: string
	hideRemove?: boolean
}>()

function emitRemove() {
	emit('remove')
}
</script>
