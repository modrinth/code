<template>
	<div class="contents">
		<div class="flex flex-row items-center gap-2 rounded-lg">
			<ButtonStyled v-if="isInstalling" type="standard" color="brand" size="large">
				<button disabled class="flex-shrink-0">
					<LoaderCircleIcon class="size-5 animate-spin" /> Installing...
				</button>
			</ButtonStyled>

			<template v-else>
				<ButtonStyled v-if="showStopButton" type="transparent" size="large">
					<button v-tooltip="busyTooltip" :disabled="!canTakeAction" @click="initiateAction('Stop')">
						<div class="flex gap-1">
							<StopCircleIcon class="h-5 w-5" />
							<span>Stop</span>
						</div>
					</button>
				</ButtonStyled>

				<ButtonStyled type="standard" color="brand" size="large">
					<button v-tooltip="busyTooltip" :disabled="!canTakeAction" @click="handlePrimaryAction">
						<component :is="isRunning || showStopButton ? UpdatedIcon : PlayIcon" />
						<span>{{ primaryActionText }}</span>
					</button>
				</ButtonStyled>
			</template>
		</div>
	</div>
</template>

<script setup lang="ts">
import { LoaderCircleIcon, PlayIcon, StopCircleIcon, UpdatedIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { ButtonStyled } from '#ui/components'

import { useServerPowerAction } from './use-server-power-action'

const props = withDefaults(
	defineProps<{
		disabled?: boolean
	}>(),
	{
		disabled: false,
	},
)

const {
	isInstalling,
	isRunning,
	showStopButton,
	busyTooltip,
	canTakeAction,
	primaryActionText,
	initiateAction,
	handlePrimaryAction,
} = useServerPowerAction({
	disabled: computed(() => props.disabled),
})
</script>
