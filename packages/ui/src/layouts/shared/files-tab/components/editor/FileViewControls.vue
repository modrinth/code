<template>
	<div
		class="flex items-center gap-2 rounded-xl border border-solid border-surface-4 bg-surface-3 p-1 shadow-sm"
		:class="{ 'pl-3': showVariants }"
	>
		<template v-if="showVariants && variants">
			<span class="text-sm font-semibold text-secondary">{{ variants.label }}</span>
			<Combobox
				class="w-36"
				trigger-size="sm"
				:aria-label="variants.label"
				:model-value="variants.selected"
				:options="variants.options.map((option) => ({ value: option.id, label: option.label }))"
				@update:model-value="(id) => id && ui.setContentVariant(tabId, file, id)"
			/>
		</template>
		<TeleportOverflowMenu
			type="quiet"
			size="sm"
			:circular="false"
			:label="formatMessage(messages.interpretation)"
			:tooltip="formatMessage(messages.interpretationTooltip, { current: currentLabel })"
			:options="options"
			placement="top-end"
		>
			<component :is="INTERPRETATION_ICONS[interpretation]" aria-hidden="true" />
		</TeleportOverflowMenu>
	</div>
</template>

<script setup lang="ts">
import { BinaryIcon, BracesIcon, ScanEyeIcon, TypeIcon } from '@modrinth/assets'
import { computed } from 'vue'

import Combobox from '#ui/components/base/Combobox.vue'
import { type ButtonMenuOption, TeleportOverflowMenu } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { injectFileBrowserUI } from '../../providers/file-browser-ui'
import type { FileInfo, FileInterpretation } from '../../providers/file-manager'

/**
 * The corner controls of a tab's file view: the content variant it's read as (e.g. which
 * decompiler disassembles a class file), and how the file is shown, as by default or as its
 * stored bytes in text, hex or JSON.
 */
const props = defineProps<{
	tabId: string
	file: FileInfo<'file'>
}>()

const { formatMessage } = useVIntl()
const ui = injectFileBrowserUI()

const messages = defineMessages({
	interpretation: { id: 'files.view-controls.interpretation', defaultMessage: 'Show as' },
	interpretationTooltip: {
		id: 'files.view-controls.interpretation-tooltip',
		defaultMessage: 'Showing as {current}',
	},
	auto: { id: 'files.view-controls.auto', defaultMessage: 'Default view' },
	autoVariant: { id: 'files.view-controls.auto-variant', defaultMessage: 'Disassembled' },
	text: { id: 'files.view-controls.text', defaultMessage: 'Text' },
	hex: { id: 'files.view-controls.hex', defaultMessage: 'Hex' },
	json: { id: 'files.view-controls.json', defaultMessage: 'JSON' },
})

const INTERPRETATION_ICONS = {
	auto: ScanEyeIcon,
	text: TypeIcon,
	hex: BinaryIcon,
	json: BracesIcon,
} satisfies Record<FileInterpretation, unknown>

const variants = computed(() => ui.contentVariantOf(props.tabId, props.file))
const interpretation = computed(() => ui.interpretationOf(props.tabId, props.file))
const showVariants = computed(() => !!variants.value && interpretation.value === 'auto')

function labelOf(value: FileInterpretation) {
	if (value === 'auto') return formatMessage(variants.value ? messages.autoVariant : messages.auto)
	return formatMessage(messages[value])
}

const currentLabel = computed(() => labelOf(interpretation.value))

const options = computed<ButtonMenuOption[]>(() =>
	(['auto', 'text', 'hex', 'json'] as const).map((value) => ({
		id: value,
		label: labelOf(value),
		icon: INTERPRETATION_ICONS[value],
		selected: interpretation.value === value,
		action: () => ui.setInterpretation(props.tabId, props.file, value),
	})),
)
</script>
