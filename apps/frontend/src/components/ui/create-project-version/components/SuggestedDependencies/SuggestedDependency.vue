<template>
	<div
		class="flex items-center justify-between gap-2 rounded-xl bg-button-bg px-4 py-1 text-button-text"
	>
		<div class="flex items-center gap-2">
			<Avatar v-if="icon" :src="icon" alt="dependency-icon" size="20px" />
			<span class="overflow-hidden font-semibold text-contrast" :title="name || 'Unknown Project'">
				{{ name || 'Unknown Project' }}
			</span>

			<TagItem class="border !border-solid border-surface-5">
				{{ dependencyType }}
			</TagItem>
		</div>

		<span
			v-if="versionName"
			class="overflow-hidden text-ellipsis whitespace-nowrap font-medium"
			:title="versionName"
		>
			{{ versionName }}
		</span>

		<div class="flex items-center gap-1">
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
	dependencyType: Labrinth.Versions.v3.DependencyType
	versionName?: string
}>()

function emitAddSuggestion() {
	emit('onAddSuggestion')
}
</script>
