<template>
	<div class="contents">
		<div class="flex flex-row items-center gap-2 rounded-lg">
			<Button type="colored" color="brand" size="xl" v-if="isInstalling" disabled class="flex-shrink-0">
				<LoaderCircleIcon class="size-5 animate-spin" /> Installing...
			</Button>

			<template v-else-if="showRestartButton">
				<Button type="colored" color="orange" size="xl" v-tooltip="busyTooltip" :disabled="!canTakeAction" @click="handlePrimaryAction">
					<UpdatedIcon />
					<span>{{ primaryActionText }}</span>
				</Button>

				<SplitButton
					type="colored"
					color="red"
					size="xl"
					:options="stopMenuOptions"
					menu-label="Force stop options"
					:primary-disabled="!canTakeAction"
					:menu-disabled="!canKill"
					v-tooltip="busyTooltip"
					@click="initiateAction('Stop')"
				>
					<StopCircleIcon />
					{{ isStopping ? 'Stopping' : 'Stop' }}
				</SplitButton>
			</template>

			<template v-else-if="isStopping">
				<SplitButton
					type="colored"
					color="red"
					size="xl"
					:options="stopMenuOptions"
					menu-label="Force stop options"
					:primary-disabled="true"
					:menu-disabled="!canKill"
					class="opacity-60"
					v-tooltip="busyTooltip"
				>
					<StopCircleIcon />
					Stopping
				</SplitButton>
			</template>

			<template v-else>
				<Button type="colored" color="brand" size="xl" v-tooltip="busyTooltip" :disabled="!canTakeAction" @click="handlePrimaryAction">
					<PlayIcon />
					<span>{{ primaryActionText }}</span>
				</Button>
			</template>
		</div>
	</div>
</template>

<script setup lang="ts">
import { Button, SplitButton } from '#ui/components/base/buttons'
import type { OverflowMenuOption } from '#ui/components/base/buttons'
import {
	LoaderCircleIcon,
	PlayIcon,
	SlashIcon,
	StopCircleIcon,
	UpdatedIcon,
} from '@modrinth/assets'
import { computed } from 'vue'

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

const stopMenuOptions = computed<OverflowMenuOption[]>(() => [
	{
		id: 'kill_server',
		label: 'Kill server',
		icon: SlashIcon,
		action: () => initiateAction('Kill'),
	},
])
</script>
