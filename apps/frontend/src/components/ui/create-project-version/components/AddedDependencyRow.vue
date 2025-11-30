<template>
	<div
		class="flex items-center justify-between gap-2 rounded-xl bg-button-bg px-4 py-2 text-button-text"
	>
		<div class="flex items-center gap-2">
			<Avatar v-if="icon" :src="icon" alt="dependency-icon" size="sm" />
			<span class="overflow-hidden text-ellipsis whitespace-nowrap font-medium">{{ name }}</span>
			<TagItem>{{ dependencyType }}</TagItem>
		</div>

		<span class="font-medium">{{ versionName }}</span>

		<div class="flex items-center gap-1">
			<ButtonStyled size="standard">
				<button aria-label="Remove file" @click="emitRemove">
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

const { name, icon, dependencyType, versionName } = defineProps<{
	name: string
	icon?: string
	dependencyType: Labrinth.Versions.v3.DependencyType
	versionName: string
}>()

function emitRemove() {
	emit('remove')
}
</script>
