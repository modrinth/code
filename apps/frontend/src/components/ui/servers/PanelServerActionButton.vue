<template>
	<div class="contents">
		<NewModal ref="confirmActionModal" header="Confirming power action" @close="resetPowerAction">
			<div class="flex flex-col gap-4 md:w-[400px]">
				<p class="m-0">
					Are you sure you want to
					<span class="lowercase">{{ pendingAction }}</span> the server?
				</p>
				<Checkbox
					v-model="dontAskAgain"
					label="Don't ask me again"
					class="text-sm"
					:disabled="!pendingAction"
				/>
				<div class="flex flex-row gap-4">
					<ButtonStyled type="standard" color="brand" @click="executePowerAction">
						<button>
							<CheckIcon class="h-5 w-5" />
							{{ pendingAction }} server
						</button>
					</ButtonStyled>
					<ButtonStyled @click="resetPowerAction">
						<button>
							<XIcon class="h-5 w-5" />
							Cancel
						</button>
					</ButtonStyled>
				</div>
			</div>
		</NewModal>

		<NewModal
			ref="detailsModal"
			:header="`All of ${server.name || 'Server'} info`"
			@close="detailsModal?.hide()"
		>
			<ServerInfoLabels
				:server-data="server"
				:show-game-label="true"
				:show-loader-label="true"
				:uptime-seconds="uptimeSeconds"
				:column="true"
				class="mb-6 flex flex-col gap-2"
			/>
			<div v-if="flags.advancedDebugInfo" class="markdown-body">
				<pre>{{ server }}</pre>
			</div>
			<ButtonStyled type="standard" color="brand" @click="detailsModal?.hide()">
				<button class="w-full">Close</button>
			</ButtonStyled>
		</NewModal>

		<div class="flex flex-row items-center gap-2 rounded-lg">
			<ButtonStyled v-if="isInstalling" type="standard" color="brand" size="large">
				<button disabled class="flex-shrink-0">
					<PanelSpinner class="size-5" /> Installing...
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
					<button
						v-tooltip="busyTooltip"
						:disabled="!canTakeAction"
						@click="handlePrimaryAction"
					>
						<div v-if="isTransitioning" class="grid place-content-center">
							<LoadingIcon />
						</div>
						<component :is="isRunning ? UpdatedIcon : PlayIcon" v-else />
						<span>{{ primaryActionText }}</span>
					</button>
				</ButtonStyled>

				<ButtonStyled circular type="transparent" size="large">
					<TeleportOverflowMenu :options="[...menuOptions]">
						<MoreVerticalIcon aria-hidden="true" />
						<template #kill>
							<SlashIcon class="h-5 w-5" />
							<span>Kill server</span>
						</template>
						<template #allServers>
							<ServerIcon class="h-5 w-5" />
							<span>All servers</span>
						</template>
						<template #details>
							<InfoIcon class="h-5 w-5" />
							<span>Details</span>
						</template>
						<template #copy-id>
							<ClipboardCopyIcon class="h-5 w-5" aria-hidden="true" />
							<span>Copy ID</span>
						</template>
					</TeleportOverflowMenu>
				</ButtonStyled>
			</template>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	CheckIcon,
	ClipboardCopyIcon,
	InfoIcon,
	MoreVerticalIcon,
	PlayIcon,
	ServerIcon,
	SlashIcon,
	StopCircleIcon,
	UpdatedIcon,
	XIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	Checkbox,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	NewModal,
	ServerInfoLabels,
	useVIntl,
} from '@modrinth/ui'
import { useStorage } from '@vueuse/core'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import LoadingIcon from './icons/LoadingIcon.vue'
import PanelSpinner from './PanelSpinner.vue'
import TeleportOverflowMenu from './TeleportOverflowMenu.vue'

type PowerAction = 'Start' | 'Stop' | 'Restart' | 'Kill'

const props = defineProps<{
	disabled?: boolean
	uptimeSeconds: number
}>()

const { formatMessage } = useVIntl()
const flags = useFeatureFlags()
const router = useRouter()
const client = injectModrinthClient()
const { serverId, server, powerState, busyReasons } = injectModrinthServerContext()
const { addNotification } = injectNotificationManager()

const confirmActionModal = ref<InstanceType<typeof NewModal> | null>(null)
const detailsModal = ref<InstanceType<typeof NewModal> | null>(null)
const pendingAction = ref<PowerAction | null>(null)
const dontAskAgain = ref(false)

const userPreferences = useStorage(`pyro-server-${serverId}-preferences`, {
	powerDontAskAgain: false,
})

const isInstalling = computed(() => server.value.status === 'installing')
const isRunning = computed(() => powerState.value === 'running')
const isStopping = computed(() => powerState.value === 'stopping')
const isTransitioning = computed(
	() => powerState.value === 'starting' || powerState.value === 'stopping',
)
const showStopButton = computed(() => isRunning.value || isStopping.value)

const busyTooltip = computed(() =>
	busyReasons.value.length > 0 ? formatMessage(busyReasons.value[0].reason) : undefined,
)

const canTakeAction = computed(
	() => !isTransitioning.value && !props.disabled && busyReasons.value.length === 0,
)

const primaryActionText = computed(() => {
	switch (powerState.value) {
		case 'starting':
			return 'Starting...'
		case 'stopping':
			return 'Stopping...'
		case 'running':
			return 'Restart'
		default:
			return 'Start'
	}
})

const menuOptions = computed(() => [
	...(isInstalling.value
		? []
		: [
				{
					id: 'kill',
					label: 'Kill server',
					icon: SlashIcon,
					action: () => initiateAction('Kill'),
				},
			]),
	{
		id: 'allServers',
		label: 'All servers',
		icon: ServerIcon,
		action: () => router.push('/hosting/manage'),
	},
	{
		id: 'details',
		label: 'Details',
		icon: InfoIcon,
		action: () => detailsModal.value?.show(),
	},
	{
		id: 'copy-id',
		label: 'Copy ID',
		icon: ClipboardCopyIcon,
		action: () => copyId(),
		shown: flags.value.developerMode,
	},
])

async function copyId() {
	await navigator.clipboard.writeText(serverId)
}

async function sendPowerAction(action: PowerAction) {
	try {
		await client.archon.servers_v0.power(serverId, action)
	} catch (error) {
		console.error(`Error performing ${action} on server:`, error)
		addNotification({
			type: 'error',
			title: `Failed to ${action.toLowerCase()} server`,
			text: 'An error occurred while performing this action.',
		})
	}
}

function initiateAction(action: PowerAction) {
	if (!canTakeAction.value) return

	if (action === 'Start') {
		sendPowerAction(action)
		return
	}

	pendingAction.value = action

	if (userPreferences.value.powerDontAskAgain) {
		executePowerAction()
	} else {
		confirmActionModal.value?.show()
	}
}

function handlePrimaryAction() {
	initiateAction(isRunning.value ? 'Restart' : 'Start')
}

function executePowerAction() {
	if (!pendingAction.value) return

	sendPowerAction(pendingAction.value)

	if (dontAskAgain.value) {
		userPreferences.value.powerDontAskAgain = true
	}

	resetPowerAction()
}

function resetPowerAction() {
	confirmActionModal.value?.hide()
	pendingAction.value = null
	dontAskAgain.value = false
}
</script>
