<template>
	<div
		class="flex items-center justify-between gap-2 rounded-xl bg-button-bg px-4 py-2 text-button-text"
	>
		<div class="flex items-center gap-2 overflow-hidden">
			<div class="grid h-5 w-5 place-content-center rounded-full bg-green">
				<CheckIcon class="text-sm text-black" />
			</div>
			<span class="overflow-hidden text-ellipsis whitespace-nowrap font-medium" :title="name">
				{{ name }}
			</span>
			<div
				v-if="isPrimary"
				class="rounded-full border border-solid border-brand bg-highlight-green px-2.5 py-1 text-sm font-medium text-brand"
			>
				Primary
			</div>
		</div>

		<div class="flex items-center gap-1">
			<template v-if="!isPrimary">
				<div class="w-36">
					<Combobox
						v-model="selectedType"
						:searchable="false"
						class="rounded-xl border border-solid border-surface-5 text-sm"
						:options="versionTypes"
						:close-on-select="true"
						:show-labels="false"
						:allow-empty="false"
						@update:model-value="emitFileTypeChange"
					/>
				</div>
			</template>

			<ButtonStyled v-if="onRemove" size="standard" :circular="true">
				<button aria-label="Remove file" class="!shadow-none" @click="onRemove">
					<XIcon aria-hidden="true" />
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CheckIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, Combobox } from '@modrinth/ui'
import { type DropdownOption } from '@modrinth/ui/src/components/base/Combobox.vue'

const emit = defineEmits<{
	(e: 'setPrimaryFile'): void
	(e: 'setFileType', type: Labrinth.Versions.v3.FileType): void
}>()

const { name, isPrimary, onRemove, initialFileType, editingVersion } = defineProps<{
	name: string
	isPrimary: boolean
	onRemove?: () => void
	initialFileType?: Labrinth.Versions.v3.FileType | 'primary'
	editingVersion: boolean
}>()

const selectedType = ref<Labrinth.Versions.v3.FileType | 'primary'>(initialFileType || 'unknown')

const versionTypes = [
	!editingVersion && { class: 'text-sm', value: 'primary', label: 'Primary' },
	{ class: 'text-sm', value: 'unknown', label: 'Other' },
	{ class: 'text-sm', value: 'required-resource-pack', label: 'Required RP' },
	{ class: 'text-sm', value: 'optional-resource-pack', label: 'Optional RP' },
	{ class: 'text-sm', value: 'sources-jar', label: 'Sources JAR' },
	{ class: 'text-sm', value: 'dev-jar', label: 'Dev JAR' },
	{ class: 'text-sm', value: 'javadoc-jar', label: 'Javadoc JAR' },
	{ class: 'text-sm', value: 'signature', label: 'Signature' },
].filter(Boolean) as DropdownOption<Labrinth.Versions.v3.FileType | 'primary'>[]

function emitFileTypeChange() {
	if (selectedType.value === 'primary') emit('setPrimaryFile')
	else emit('setFileType', selectedType.value)
}
</script>
