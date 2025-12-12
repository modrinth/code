<template>
	<div
		:class="[
			'grid items-center justify-between gap-2 rounded-xl bg-button-bg px-4 py-1 text-button-text',
			versionName ? 'grid-cols-[2fr_1fr_auto]' : 'grid-cols-[2fr_auto]',
		]"
	>
		<div class="flex items-center gap-2">
			<Avatar v-if="icon" :src="icon" alt="dependency-icon" size="20px" :no-shadow="true" />

			<span class="text-wrap font-semibold text-contrast" :title="name || projectId">
				{{ name || 'Unknown Project' }}
			</span>

			<TagItem class="shrink-0 border !border-solid border-surface-5">
				{{ dependencyType }}
			</TagItem>
		</div>

		<span v-if="versionName" class="truncate whitespace-nowrap font-medium" :title="versionName">
			{{ versionName }}
		</span>

		<div class="flex items-center justify-end gap-1">
			<ButtonStyled size="standard" :circular="true">
				<button aria-label="Remove file" class="!shadow-none" @click="emitRemove">
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

const { projectId, name, icon, dependencyType, versionName } = defineProps<{
	projectId: string
	name?: string
	icon?: string
	dependencyType: Labrinth.Versions.v3.DependencyType
	versionName?: string
}>()

function emitRemove() {
	emit('remove')
}
</script>
