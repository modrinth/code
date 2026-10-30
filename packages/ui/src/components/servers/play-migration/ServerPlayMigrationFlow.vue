<template>
	<div class="flex w-[496px] max-w-full flex-col">
		<ServerPlayMigrationMrpackStage
			v-if="stage === 'upload' && kind === 'mrpack'"
			:disabled="disabled"
			@back="stage = 'intro'"
			@confirm="emit('confirm', { kind: 'mrpack', file: $event })"
		/>
		<ServerPlayMigrationCurseForgeStage
			v-else-if="stage === 'upload' && kind === 'curseforge'"
			:selection="selection"
			:disabled="disabled"
			@back="stage = 'intro'"
			@confirm="emit('confirm', { kind: 'curseforge', selection })"
		/>
		<ServerPlayMigrationContent
			v-else
			v-model:acknowledged="acknowledged"
			:modpack-name="kind === 'modrinth' ? modpackName : null"
			:cancel="cancel"
			:disabled="disabled"
			:continues="needsUpload"
			@back="emit('back')"
			@confirm="confirmIntro"
		/>
	</div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

import { createCurseForgeSelection } from '#ui/components/flows/creation-flow-modal/curseforge'
import { injectModrinthClient } from '#ui/providers'

import ServerPlayMigrationContent from './ServerPlayMigrationContent.vue'
import ServerPlayMigrationCurseForgeStage from './ServerPlayMigrationCurseForgeStage.vue'
import ServerPlayMigrationMrpackStage from './ServerPlayMigrationMrpackStage.vue'
import type { ServerPlayMigrationKind, ServerPlayMigrationSource } from './types'

const props = defineProps<{
	kind: ServerPlayMigrationKind | null
	modpackName?: string | null
	hasUploadedMrpack?: boolean
	worldId: string | null
	cancel?: boolean
	disabled?: boolean
}>()

const emit = defineEmits<{
	back: []
	confirm: [source: ServerPlayMigrationSource]
}>()

const client = injectModrinthClient()
const stage = ref<'intro' | 'upload'>('intro')
const acknowledged = ref(false)
const selection = createCurseForgeSelection(client, () => {
	if (!props.worldId) throw new Error('The server has no world to restore')
	return { worldId: props.worldId }
})

const needsUpload = computed(
	() => props.kind === 'curseforge' || (props.kind === 'mrpack' && !props.hasUploadedMrpack),
)

function confirmIntro() {
	if (needsUpload.value) {
		stage.value = 'upload'
		return
	}
	emit('confirm', props.kind === 'mrpack' ? { kind: 'mrpack', file: null } : { kind: 'restore' })
}
</script>
