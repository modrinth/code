<script setup lang="ts" generic="T">
import { HistoryIcon, SaveIcon, SpinnerIcon } from '@modrinth/assets'
import { type Component, computed } from 'vue'

import { defineMessage, type MessageDescriptor, useVIntl } from '../../composables/i18n'
import { commonMessages } from '../../utils'
import ButtonStyled from './ButtonStyled.vue'
import FloatingActionBar from './FloatingActionBar.vue'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
	(e: 'reset' | 'save', event: MouseEvent): void
}>()

const props = withDefaults(
	defineProps<{
		canReset?: boolean
		original: T
		modified: Partial<T>
		saving?: boolean
		text?: MessageDescriptor | string
		saveLabel?: MessageDescriptor | string
		savingLabel?: MessageDescriptor | string
		saveIcon?: Component
	}>(),
	{
		canReset: true,
		saving: false,
		text: () =>
			defineMessage({
				id: 'ui.component.unsaved-changes-popup.body',
				defaultMessage: 'You have unsaved changes.',
			}),
		saveLabel: () => commonMessages.saveButton,
		savingLabel: () => commonMessages.savingButton,
		saveIcon: SaveIcon,
	},
)

const shown = computed(() => {
	let changed = false
	for (const key of Object.keys(props.modified)) {
		if (props.original[key] !== props.modified[key]) {
			changed = true
		}
	}
	return changed
})

function localizeIfPossible(message: MessageDescriptor | string) {
	return typeof message === 'string' ? message : formatMessage(message)
}
</script>

<template>
	<FloatingActionBar :shown="shown">
		<p class="m-0 font-semibold text-sm md:text-base">{{ localizeIfPossible(text) }}</p>
		<div class="ml-auto flex gap-2">
			<ButtonStyled v-if="canReset" type="transparent">
				<button :disabled="saving" @click="(e) => emit('reset', e)">
					<HistoryIcon /> {{ formatMessage(commonMessages.resetButton) }}
				</button>
			</ButtonStyled>
			<ButtonStyled color="brand">
				<button :disabled="saving" @click="(e) => emit('save', e)">
					<SpinnerIcon v-if="saving" class="animate-spin" />
					<component :is="saveIcon" v-else />
					{{ localizeIfPossible(saving ? savingLabel : saveLabel) }}
				</button>
			</ButtonStyled>
		</div>
	</FloatingActionBar>
</template>
