<template>
	<div
		class="flex items-center justify-between gap-2 rounded-xl bg-button-bg px-4 py-2 text-button-text"
	>
		<div class="flex items-center gap-2 overflow-hidden">
			<div class="grid h-5 w-5 place-content-center rounded-full bg-green">
				<CheckIcon class="text-sm text-black" />
			</div>
			<span class="overflow-hidden text-ellipsis whitespace-nowrap font-medium" :title="file.name">
				{{ file.name }}
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
				<div class="w-28">
					<Combobox
						v-model="selectedType"
						:searchable="false"
						class="rounded-xl border border-solid border-surface-5"
						:options="versionTypes"
						:close-on-select="true"
						:show-labels="false"
						:allow-empty="false"
						@update:model-value="emitFileTypeChange"
					/>
				</div>
			</template>

			<ButtonStyled size="standard" :circular="true">
				<button aria-label="Remove file" class="!shadow-none" @click="emitRemove">
					<XIcon aria-hidden="true" />
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import { CheckIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled } from '@modrinth/ui'
import Combobox, { type DropdownOption } from '@modrinth/ui/src/components/base/Combobox.vue'

const selectedType = ref<string>('other')

const emit = defineEmits<{
	(e: 'setPrimaryFile' | 'remove'): void
}>()

const { file, isPrimary } = defineProps<{
	file: File
	isPrimary: boolean
}>()

const versionTypes: DropdownOption<string>[] = [
	{ value: 'primary', label: 'Primary' },
	{ value: 'other', label: 'Other' },
]

function emitFileTypeChange() {
	if (selectedType.value) emit('setPrimaryFile')
	selectedType.value = isPrimary ? 'primary' : 'other'
}

function emitRemove() {
	emit('remove')
}
</script>
