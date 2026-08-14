<script setup lang="ts" generic="T">
import { HistoryIcon, SaveIcon, SpinnerIcon } from '@modrinth/assets'
import { isEqual } from 'es-toolkit'
import { type Component, computed, ref } from 'vue'

import { Button } from '#ui/components/base/buttons'

import { defineMessage, type MessageDescriptor, useVIntl } from '../../composables/i18n'
import { commonMessages } from '../../utils'
import FloatingActionBar from './FloatingActionBar.vue'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
	(e: 'reset' | 'save', event: MouseEvent): void
}>()

type SaveDisabledReason = MessageDescriptor | string

const props = withDefaults(
	defineProps<{
		canReset?: boolean
		canSave?: boolean
		original: T
		modified: Partial<T>
		saving?: boolean
		text?: MessageDescriptor | string
		saveLabel?: MessageDescriptor | string
		savingLabel?: MessageDescriptor | string
		saveDisabledReason?: SaveDisabledReason | SaveDisabledReason[]
		saveIcon?: Component
		inline?: boolean
	}>(),
	{
		canReset: true,
		canSave: true,
		saving: false,
		text: () =>
			defineMessage({
				id: 'ui.component.unsaved-changes-popup.body',
				defaultMessage: 'You have unsaved changes.',
			}),
		saveLabel: () => commonMessages.saveButton,
		savingLabel: () => commonMessages.savingButton,
		saveDisabledReason: undefined,
		saveIcon: SaveIcon,
		inline: false,
	},
)

const shown = computed(
	() =>
		props.saving ||
		Object.keys(props.modified).some((key) => !isEqual(props.original[key], props.modified[key])),
)

function localizeIfPossible(message: MessageDescriptor | string) {
	return typeof message === 'string' ? message : formatMessage(message)
}

const saveDisabled = computed(() => props.saving || !props.canSave)

const saveDisabledTooltip = computed(() => {
	if (!saveDisabled.value || props.saving || !props.saveDisabledReason) {
		return undefined
	}

	const reasons = (
		Array.isArray(props.saveDisabledReason) ? props.saveDisabledReason : [props.saveDisabledReason]
	).map(localizeIfPossible)

	return reasons.length > 0
		? { content: reasons.join('\n'), popperClass: 'unsaved-changes-save-tooltip' }
		: undefined
})

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
				<HistoryIcon /> {{ formatMessage(commonMessages.resetButton) }}
			</Button>
			<span
				v-tooltip="saveDisabledTooltip"
				class="flex"
				:class="{ 'cursor-not-allowed': saveDisabled }"
			>
				<Button
					type="colored"
					color="brand"
					:disabled="saveDisabled"
					:class="{ 'pointer-events-none': saveDisabled }"
					@click="(e) => emit('save', e)"
				>
					<SpinnerIcon v-if="saving" class="animate-spin" />
					<component :is="saveIcon" v-else />
					{{ localizeIfPossible(saving ? savingLabel : saveLabel) }}
				</Button>
			</span>
		</div>
	</FloatingActionBar>
</template>

<style lang="scss">
/* unscoped because floating-vue teleports the tooltip outside of scope */
.v-popper__popper.v-popper--theme-tooltip.unsaved-changes-save-tooltip .v-popper__inner {
	max-width: 22rem;
	white-space: pre-line;
	line-height: 1.4;
}
</style>
