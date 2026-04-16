<template>
	<div class="contents">
		<div class="flex flex-row items-center gap-2 rounded-lg">
			<ButtonStyled v-if="isInstalling" type="standard" color="brand" size="large">
				<button disabled class="flex-shrink-0">
					<LoaderCircleIcon class="size-5 animate-spin" /> Installing...
				</button>
			</ButtonStyled>

			<template v-else>
				<ButtonStyled v-if="showStopButton" type="standard" color="red" size="large">
					<button
						v-tooltip="busyTooltip"
						:disabled="!canTakeAction"
						@click="initiateAction('Stop')"
					>
						<div class="flex gap-1">
							<StopCircleIcon class="h-5 w-5" />
							<span>Stop</span>
						</div>
					</button>
				</ButtonStyled>

				<div v-if="showRestartDropdown" class="joined-buttons">
					<ButtonStyled type="standard" color="orange" size="large">
						<button v-tooltip="busyTooltip" :disabled="!canTakeAction" @click="handlePrimaryAction">
							<UpdatedIcon />
							<span>{{ primaryActionText }}</span>
						</button>
					</ButtonStyled>
					<ButtonStyled type="standard" color="orange" size="large">
						<OverflowMenu
							v-tooltip="busyTooltip"
							:disabled="!canTakeAction"
							:options="[
								{
									id: 'kill_server',
									action: () => initiateAction('Kill'),
								},
							]"
						>
							<div class="w-0 text-xl relative top-0.5 right-2.5">
								<DropdownIcon />
							</div>

							<template #kill_server>
								<SlashIcon class="h-5 w-5" />
								Kill server
							</template>
						</OverflowMenu>
					</ButtonStyled>
				</div>
				<ButtonStyled v-else type="standard" color="brand" size="large">
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
	DropdownIcon,
	LoaderCircleIcon,
	PlayIcon,
	SlashIcon,
	StopCircleIcon,
	UpdatedIcon,
} from '@modrinth/assets'
import { computed } from 'vue'

import { ButtonStyled, OverflowMenu } from '#ui/components'

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
	showStopButton,
	busyTooltip,
	canTakeAction,
	primaryActionText,
	initiateAction,
	handlePrimaryAction,
} = useServerPowerAction({
	disabled: computed(() => props.disabled),
})

const showRestartDropdown = computed(() => primaryActionText.value === 'Restart')
</script>

<style scoped>
.joined-buttons {
	display: flex;
	align-items: center;
}

.joined-buttons > :deep(.btn) {
	border-radius: 0;
}

.joined-buttons > :deep(.btn:first-child) {
	border-top-left-radius: var(--radius-md);
	border-bottom-left-radius: var(--radius-md);
}

.joined-buttons > :deep(.btn:last-child) {
	border-top-right-radius: var(--radius-md);
	border-bottom-right-radius: var(--radius-md);
	margin-left: -1px;
}

.joined-buttons > :deep(.btn:not(:last-child)) {
	border-right: none;
}
</style>
