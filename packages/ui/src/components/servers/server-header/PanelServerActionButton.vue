<template>
	<div class="contents">
		<div class="flex flex-row items-center gap-2 rounded-lg">
			<ButtonStyled v-if="isInstalling" type="standard" color="brand" :size="size">
				<button disabled class="flex-shrink-0">
					<LoaderCircleIcon class="size-5 animate-spin" /> Installing...
				</button>
			</ButtonStyled>

			<template v-else-if="showRestartButton">
				<JoinedButtons
					v-if="powerActionWorlds.length"
					color="orange"
					:size="size"
					:actions="restartSplitActions"
					:primary-disabled="!canTakeAction"
					:dropdown-disabled="!canTakeAction"
					:primary-tooltip="busyTooltip"
					:dropdown-tooltip="busyTooltip"
				/>
				<ButtonStyled v-else type="standard" color="orange" :size="size">
					<button v-tooltip="busyTooltip" :disabled="!canTakeAction" @click="handlePrimaryAction">
						<UpdatedIcon />
						<span>{{ primaryActionText }}</span>
					</button>
				</ButtonStyled>

				<JoinedButtons
					color="red"
					:size="size"
					:actions="stopSplitActions"
					:primary-disabled="!canTakeAction"
					:dropdown-disabled="!canKill"
					:primary-tooltip="busyTooltip"
					:dropdown-tooltip="busyTooltip"
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
					:size="size"
					:actions="stopSplitActions"
					:primary-disabled="true"
					:dropdown-disabled="!canKill"
					:primary-muted="true"
					:dropdown-tooltip="busyTooltip"
				>
					<template #kill_server>
						<SlashIcon class="h-5 w-5" />
						Kill server
					</template>
				</JoinedButtons>
			</template>

			<template v-else>
				<JoinedButtons
					v-if="powerActionWorlds.length"
					color="brand"
					:size="size"
					:actions="startSplitActions"
					:primary-disabled="!canTakeAction"
					:dropdown-disabled="!canTakeAction"
					:primary-tooltip="busyTooltip"
					:dropdown-tooltip="busyTooltip"
				/>
				<ButtonStyled v-else type="standard" color="brand" :size="size">
					<button v-tooltip="busyTooltip" :disabled="!canTakeAction" @click="handlePrimaryAction">
						<PlayIcon />
						<span>{{ startActionText }}</span>
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
		size?: 'standard' | 'large' | 'small'
		startLabel?: string
		worlds?: { id: string; name: string }[]
	}>(),
	{
		disabled: false,
		size: 'large',
		startLabel: 'Start',
		worlds: () => [],
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

const size = computed(() => props.size)
const startActionText = computed(() =>
	primaryActionText.value === 'Start' ? props.startLabel : primaryActionText.value,
)
const powerActionWorlds = computed(() => (props.worlds.length > 1 ? props.worlds : []))

const startSplitActions = computed<JoinedButtonAction[]>(() => [
	{
		id: 'start',
		label: startActionText.value,
		icon: PlayIcon,
		action: handlePrimaryAction,
	},
	...powerActionWorlds.value.map((world) => ({
		id: `start-${world.id}`,
		label: `Start with ${world.name}`,
		icon: PlayIcon,
		action: () => initiateAction('Start', world.id),
	})),
])

const restartSplitActions = computed<JoinedButtonAction[]>(() => [
	{
		id: 'restart',
		label: primaryActionText.value,
		icon: UpdatedIcon,
		action: handlePrimaryAction,
	},
	...powerActionWorlds.value.map((world) => ({
		id: `restart-${world.id}`,
		label: `Restart with ${world.name}`,
		icon: UpdatedIcon,
		action: () => initiateAction('Restart', world.id),
	})),
])

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
