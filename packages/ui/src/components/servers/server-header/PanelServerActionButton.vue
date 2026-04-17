<template>
	<div class="contents">
		<div class="flex flex-row items-center gap-2 rounded-lg">
			<ButtonStyled v-if="isInstalling" type="standard" color="brand" size="large">
				<button disabled class="flex-shrink-0">
					<LoaderCircleIcon class="size-5 animate-spin" /> Installing...
				</button>
			</ButtonStyled>

			<template v-else-if="showRestartButton">
				<ButtonStyled type="standard" color="orange" size="large">
					<button v-tooltip="busyTooltip" :disabled="!canTakeAction" @click="handlePrimaryAction">
						<UpdatedIcon />
						<span>{{ primaryActionText }}</span>
					</button>
				</ButtonStyled>

				<JoinedButtons
					color="red"
					size="large"
					:actions="stopSplitActions"
					:primary-disabled="!canTakeAction"
					:dropdown-disabled="!canKill"
				>
					<template #kill_server>
						<SlashIcon class="h-5 w-5" />
						Kill server
					</template>
				</JoinedButtons>
			</template>

			<template v-else-if="isStopping">
				<JoinedButtons
					color="red"
					size="large"
					:actions="stopSplitActions"
					:primary-disabled="true"
					:dropdown-disabled="!canKill"
					:primary-muted="true"
				>
					<template #kill_server>
						<SlashIcon class="h-5 w-5" />
						Kill server
					</template>
				</JoinedButtons>
			</template>

			<template v-else>
				<ButtonStyled type="standard" color="brand" size="large">
					<button v-tooltip="busyTooltip" :disabled="!canTakeAction" @click="handlePrimaryAction">
						<PlayIcon />
						<span>{{ primaryActionText }}</span>
					</button>
				</ButtonStyled>
			</template>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	LoaderCircleIcon,
	PlayIcon,
	SlashIcon,
	StopCircleIcon,
	UpdatedIcon,
} from '@modrinth/assets'
import { computed } from 'vue'

import { ButtonStyled, type JoinedButtonAction, JoinedButtons } from '#ui/components'

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
	isStopping,
	showRestartButton,
	busyTooltip,
	canTakeAction,
	canKill,
	primaryActionText,
	initiateAction,
	handlePrimaryAction,
} = useServerPowerAction({
	disabled: computed(() => props.disabled),
})

const stopSplitActions = computed<JoinedButtonAction[]>(() => [
	{
		id: 'stop',
		label: isStopping.value ? 'Stopping' : 'Stop',
		icon: StopCircleIcon,
		action: () => initiateAction('Stop'),
	},
	{
		id: 'kill_server',
		label: 'Kill server',
		icon: SlashIcon,
		action: () => initiateAction('Kill'),
	},
])
</script>
