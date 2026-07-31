<template>
	<div class="contents">
		<div class="flex flex-row items-center gap-2 rounded-lg">
			<Button v-if="isInstalling" type="colored" color="brand" size="lg" disabled>
				<LoaderCircleIcon class="size-5 animate-spin" aria-hidden="true" /> Installing...
			</Button>

			<template v-else-if="showRestartButton">
				<Button
					v-tooltip="busyTooltip"
					type="colored"
					color="orange"
					size="lg"
					:disabled="!canTakeAction"
					@click="handlePrimaryAction"
				>
					<UpdatedIcon aria-hidden="true" />
					<span>{{ primaryActionText }}</span>
				</Button>

				<SplitButton
					v-tooltip="busyTooltip"
					:menu-label="formatMessage(messages.serverPowerOptions)"
					type="colored"
					color="red"
					size="lg"
					:options="stopSplitOptions"
					:primary-disabled="!canTakeAction"
					:menu-disabled="!canKill"
					@click="initiateAction('Stop')"
				>
					<StopCircleIcon aria-hidden="true" />
					{{ isStopping ? 'Stopping' : 'Stop' }}
					<template #kill_server>
						<SlashIcon class="h-5 w-5" aria-hidden="true" />
						Kill server
					</template>
				</SplitButton>
			</template>

			<template v-else-if="isStopping">
				<SplitButton
					v-tooltip="busyTooltip"
					:menu-label="formatMessage(messages.serverPowerOptions)"
					type="colored"
					color="red"
					size="lg"
					:options="stopSplitOptions"
					:primary-disabled="true"
					:menu-disabled="!canKill"
				>
					<StopCircleIcon aria-hidden="true" />
					Stopping
					<template #kill_server>
						<SlashIcon class="h-5 w-5" aria-hidden="true" />
						Kill server
					</template>
				</SplitButton>
			</template>

			<template v-else>
				<Button
					v-tooltip="busyTooltip"
					type="colored"
					color="brand"
					size="lg"
					:disabled="!canTakeAction"
					@click="handlePrimaryAction"
				>
					<PlayIcon aria-hidden="true" />
					<span>{{ primaryActionText }}</span>
				</Button>
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

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import Button from '@modrinth/ui/src/components/base/buttons/Button.vue'
import SplitButton from '@modrinth/ui/src/components/base/buttons/SplitButton.vue'
import type { OverflowMenuOption } from '@modrinth/ui/src/components/base/buttons/types'

import { useServerPowerAction } from './use-server-power-action'

const props = withDefaults(
	defineProps<{
		disabled?: boolean
	}>(),
	{
		disabled: false,
	},
)

const { formatMessage } = useVIntl()
const messages = defineMessages({
	serverPowerOptions: {
		id: 'servers.power.options',
		defaultMessage: 'Server power options',
	},
})

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

const stopSplitOptions = computed<OverflowMenuOption[]>(() => [
	{
		id: 'kill_server',
		label: 'Kill server',
		icon: SlashIcon,
		action: () => initiateAction('Kill'),
	},
])
</script>
