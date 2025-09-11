<script setup lang="ts" generic="T">
import { HistoryIcon, SaveIcon, SpinnerIcon } from '@modrinth/assets'
import { defineMessage, type MessageDescriptor, useVIntl } from '@vintl/vintl'
import { type Component, computed } from 'vue'

import { commonMessages } from '../../utils'
import ButtonStyled from './ButtonStyled.vue'

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
	<Transition name="pop-in">
		<div v-if="shown" class="fixed w-full z-10 left-0 p-4 unsaved-changes-popup">
			<div
				class="flex items-center gap-2 rounded-2xl bg-bg-raised border-2 border-divider border-solid mx-auto max-w-[77rem] p-4"
			>
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
			</div>
		</div>
	</Transition>
</template>

<style scoped>
.pop-in-enter-active {
	transition: all 0.5s cubic-bezier(0.15, 1.4, 0.64, 0.96);
}

.pop-in-leave-active {
	transition: all 0.25s ease;
}

.pop-in-enter-from {
	scale: 0.5;
	translate: 0 10rem;
	opacity: 0;
}

.pop-in-leave-to {
	scale: 0.96;
	translate: 0 0.25rem;
	opacity: 0;
}

.unsaved-changes-popup {
	transition: bottom 0.25s ease-in-out;
	bottom: 0;
}

@media (any-hover: none) and (max-width: 640px) {
	.unsaved-changes-popup {
		bottom: var(--size-mobile-navbar-height);
	}

	.expanded-mobile-nav .unsaved-changes-popup {
		bottom: var(--size-mobile-navbar-height-expanded);
	}
}
</style>
