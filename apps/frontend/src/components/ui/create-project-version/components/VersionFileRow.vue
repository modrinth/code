<template>
	<div
		class="flex items-center justify-between gap-2 rounded-xl bg-button-bg px-4 py-2 text-button-text"
	>
		<div class="flex items-center gap-2">
			<div class="grid h-5 w-5 place-content-center rounded-full bg-green">
				<CheckIcon class="text-md text-black" />
			</div>
			<span class="overflow-hidden text-ellipsis whitespace-nowrap font-medium">{{
				file.name
			}}</span>
		</div>

		<div class="flex items-center gap-1">
			<Combobox
				:model-value="selectedType"
				:searchable="false"
				class="w-fit rounded-xl border border-surface-5"
				:options="versionTypes"
				:close-on-select="true"
				:show-labels="false"
				:allow-empty="false"
				@update:model-value="emitFileTypeChange"
			/>
			<ButtonStyled size="standard">
				<button aria-label="Remove file" @click="emitRemove">
					<XIcon aria-hidden="true" />
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import { CheckIcon, XIcon } from '@modrinth/assets'
import ButtonStyled from '@modrinth/ui/src/components/base/ButtonStyled.vue'
import Combobox, { type DropdownOption } from '@modrinth/ui/src/components/base/Combobox.vue'

const emit = defineEmits<{
	(e: 'fileTypeChange', type: string): void
	(e: 'remove'): void
}>()

const { file, selectedType } = defineProps<{
	file: File
	selectedType: string
}>()

const versionTypes: DropdownOption<string>[] = [
	{ value: 'Primary', label: 'Primary' },
	{ value: 'Source', label: 'Source' },
	{ value: 'Other', label: 'Other' },
]

function emitFileTypeChange() {
	// const type = (event.target as HTMLSelectElement).value
	emit('fileTypeChange', selectedType)
}

function emitRemove() {
	emit('remove')
}
</script>
