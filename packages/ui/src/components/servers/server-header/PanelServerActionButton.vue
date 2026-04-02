<template>
	<div class="contents">
		<PanelActionConfirmModal
			ref="confirmActionModal"
			v-model:dont-ask-again="dontAskAgain"
			:pending-action="pendingAction"
			@confirm="executePendingAction"
			@cancel="resetPendingAction"
		/>

		<div class="flex flex-row items-center gap-2 rounded-lg">
			<ButtonStyled v-if="isInstalling" type="standard" color="brand" size="large">
				<button disabled class="flex-shrink-0">
					<LoaderCircleIcon class="size-5 animate-spin" /> Installing...
				</button>
			</ButtonStyled>

			<template v-else>
				<ButtonStyled v-if="showStopButton" type="transparent" size="large">
					<button :disabled="!canTakeAction" @click="initiateAction('Stop')">
						<div class="flex gap-1">
							<StopCircleIcon class="h-5 w-5" />
							<span>{{ isStopping ? 'Stopping...' : 'Stop' }}</span>
						</div>
					</button>
				</ButtonStyled>

				<ButtonStyled type="standard" color="brand" size="large">
					<button v-tooltip="busyTooltip" :disabled="!canTakeAction" @click="handlePrimaryAction">
						<div v-if="isTransitioning" class="grid place-content-center">
							<LoaderCircleIcon class="size-5 animate-spin" />
						</div>
						<component :is="isRunning ? UpdatedIcon : PlayIcon" v-else />
						<span>{{ primaryActionText }}</span>
					</button>
				</ButtonStyled>
			</template>
		</div>
	</div>
</template>

<script setup lang="ts">
import { LoaderCircleIcon, PlayIcon, StopCircleIcon, UpdatedIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import { ButtonStyled } from '#ui/components'
import PanelActionConfirmModal from './PanelActionConfirmModal.vue'
import {
	type PanelActionConfirmModalController,
	useServerPowerAction,
} from './use-server-power-action'

const props = withDefaults(
	defineProps<{
		disabled?: boolean
	}>(),
	{
		disabled: false,
	},
)

const confirmActionModal = ref<PanelActionConfirmModalController | null>(null)

const {
	isInstalling,
	isRunning,
	isStopping,
	isTransitioning,
	showStopButton,
	busyTooltip,
	canTakeAction,
	primaryActionText,
	pendingAction,
	dontAskAgain,
	initiateAction,
	handlePrimaryAction,
	executePendingAction,
	resetPendingAction,
} = useServerPowerAction({
	disabled: computed(() => props.disabled),
	confirmModalRef: confirmActionModal,
})
</script>
