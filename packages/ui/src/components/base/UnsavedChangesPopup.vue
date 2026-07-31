<script setup lang="ts" generic="T">
import { HistoryIcon, SaveIcon } from '@modrinth/assets'
import { isEqual } from 'es-toolkit'
import { type Component, computed, ref } from 'vue'

import { defineMessage, type MessageDescriptor, useVIntl } from '../../composables/i18n'
import { commonMessages } from '../../utils'
import Button from './buttons/Button.vue'
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
		inline?: boolean
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
		inline: false,
	},
)

const shown = computed(() =>
	Object.keys(props.modified).some((key) => !isEqual(props.original[key], props.modified[key])),
)

function localizeIfPossible(message: MessageDescriptor | string) {
	return typeof message === 'string' ? message : formatMessage(message)
}

const actionBar = ref<InstanceType<typeof FloatingActionBar> | null>(null)

function nudge(): void {
	void actionBar.value?.nudge()
}

defineExpose({ nudge })
</script>

<template>
	<FloatingActionBar ref="actionBar" :shown="shown" :inline="inline">
		<p class="m-0 font-semibold text-sm md:text-base">{{ localizeIfPossible(text) }}</p>
		<div class="ml-auto flex gap-2">
			<Button v-if="canReset" type="quiet" :disabled="saving" @click="(e) => emit('reset', e)">
				<HistoryIcon aria-hidden="true" />
				{{ formatMessage(commonMessages.resetButton) }}
			</Button>
			<Button type="colored" color="brand" :loading="saving" @click="(e) => emit('save', e)">
				<component :is="saveIcon" v-if="!saving" aria-hidden="true" />
				{{ localizeIfPossible(saving ? savingLabel : saveLabel) }}
			</Button>
		</div>
	</FloatingActionBar>
</template>
