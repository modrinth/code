<template>
	<div
		class="flex h-11 items-center justify-between gap-2 rounded-xl bg-button-bg px-4 py-1 text-button-text"
	>
		<div class="flex min-w-0 flex-1 items-center justify-start gap-2">
			<Avatar v-if="icon" :src="icon" alt="dependency-icon" size="20px" :no-shadow="true" />

			<span
				v-tooltip="name || projectId"
				class="min-w-0 max-w-fit flex-1 truncate font-semibold text-contrast"
			>
				{{ name || 'Unknown Project' }}
			</span>
			<span
				v-if="versionNumber"
				v-tooltip="versionNumber"
				class="min-w-0 max-w-fit flex-1 truncate whitespace-nowrap text-sm font-medium"
			>
				{{ versionNumber }}
			</span>

			<TagItem class="shrink-0 border !border-solid border-surface-5 capitalize">
				{{ dependencyType }}
			</TagItem>
		</div>

		<div v-if="!hideRemove" class="flex shrink-0 items-center justify-end gap-1">
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

const { projectId, name, icon, dependencyType, versionNumber, hideRemove } = defineProps<{
	projectId: string
	name?: string
	icon?: string
	dependencyType: Labrinth.Versions.v2.DependencyType
	versionNumber?: string
	hideRemove?: boolean
}>()

function emitRemove() {
	emit('remove')
}
</script>
