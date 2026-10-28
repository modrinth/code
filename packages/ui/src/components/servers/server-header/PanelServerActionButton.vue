<template>
	<div class="contents">
		<div class="flex flex-row items-center gap-2 rounded-lg">
			<Button
				v-if="isStartingRequest || sharePendingAction === 'Start'"
				type="colored"
				color="brand"
				size="xl"
				disabled
				class="flex-shrink-0"
			>
				<LoaderCircleIcon class="size-5 animate-spin" /> Starting server...
			</Button>

			<Button
				v-else-if="isInstalling"
				type="colored"
				color="brand"
				size="xl"
				disabled
				class="flex-shrink-0"
			>
				<LoaderCircleIcon class="size-5 animate-spin" /> Installing...
			</Button>

			<template v-else-if="showRestartButton">
				<Button
					v-tooltip="busyTooltip"
					type="colored"
					color="orange"
					size="xl"
					:disabled="!canTakeAction"
					@click="onPrimaryAction"
				>
					<UpdatedIcon />
					<span>{{ primaryActionText }}</span>
				</Button>

				<SplitButton
					v-tooltip="busyTooltip"
					type="colored"
					color="red"
					size="xl"
					:options="stopMenuOptions"
					menu-label="Force stop options"
					:primary-disabled="!canTakeAction"
					:menu-disabled="!canKill"
					@click="initiateAction('Stop')"
				>
					<StopCircleIcon />
					{{ isStopping ? 'Stopping' : 'Stop' }}
				</SplitButton>
			</template>

			<template v-else-if="isStopping">
				<SplitButton
					v-tooltip="busyTooltip"
					type="colored"
					color="red"
					size="xl"
					:options="stopMenuOptions"
					menu-label="Force stop options"
					:primary-disabled="true"
					:menu-disabled="!canKill"
					class="opacity-60"
				>
					<StopCircleIcon />
					Stopping
				</SplitButton>
			</template>

			<template v-else>
				<Button
					v-tooltip="busyTooltip"
					type="colored"
					color="brand"
					size="xl"
					:disabled="!canTakeAction"
					@click="onPrimaryAction"
				>
					<PlayIcon />
					<span>{{ primaryActionText }}</span>
				</Button>
			</template>
		</div>
		<ServerSharePushModal
			ref="diffModal"
			:server-id="serverId"
			:world-id="worldId ?? undefined"
			:diffs="previewQuery.data.value?.items ?? []"
			:changes-description="
				formatMessage(previewAction === 'Restart' ? messages.restartBody : messages.startBody)
			"
			:confirm-label="
				formatMessage(previewAction === 'Restart' ? messages.pushAndRestart : messages.pushAndStart)
			"
			:loading="previewQuery.isFetching.value"
			:error="previewQuery.isError.value"
			:disabled="!canSetup || previewQuery.isError.value || !previewQuery.data.value"
			@confirm="(paths) => runAction(previewAction, true, paths)"
			@cancel="previewOpen = false"
			@retry="previewQuery.refetch()"
		/>
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
import { computed, ref } from 'vue'

import type { ButtonMenuOption } from '#ui/components/base/buttons'
import { Button, SplitButton } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerPermissions } from '#ui/composables/server-permissions'
import ServerSharePushModal from '#ui/layouts/shared/server-sharing/components/ServerSharePushModal.vue'
import { useServerShareReview } from '#ui/layouts/shared/server-sharing/use-server-share-review'
import { useSharedInstanceInstalledOnce } from '#ui/layouts/wrapped/hosting/manage/[id]/play/use-server-players'
import { injectModrinthServerContext } from '#ui/providers'

import { type PowerAction, useServerPowerAction } from './use-server-power-action'

const props = withDefaults(
	defineProps<{
		disabled?: boolean
	}>(),
	{
		disabled: false,
	},
)

const { formatMessage } = useVIntl()
const { serverId, worldId, serverFull } = injectModrinthServerContext()
const { canSetup } = useServerPermissions()
const world = computed(() => serverFull.value?.worlds.find((world) => world.id === worldId.value))
const installedOnce = useSharedInstanceInstalledOnce(
	computed(() => world.value?.content?.shared_instance_id ?? null),
)
const needsShareReview = computed(
	() =>
		canSetup.value &&
		(world.value?.content?.shared_instance_needs_update ?? false) &&
		installedOnce.value,
)
const previewAction = ref<'Start' | 'Restart'>('Start')
const shareReview = useServerShareReview<'Start' | 'Restart'>({
	execute: async (action, target) => {
		await shareReview.publish(target)
		if (target.isCurrent()) await sendPowerAction(action)
	},
})
const {
	diffModal,
	previewOpen,
	previewQuery,
	actionMutation,
	pending: sharePending,
	runAction,
} = shareReview
const sharePendingAction = computed<PowerAction | undefined>(() =>
	actionMutation.isPending.value ? actionMutation.variables.value?.action : undefined,
)

const {
	isStartingRequest,
	isInstalling,
	isRunning,
	isStopping,
	showRestartButton,
	busyTooltip,
	canTakeAction,
	canKill,
	primaryActionText,
	sendPowerAction,
	initiateAction,
	handlePrimaryAction,
} = useServerPowerAction({
	disabled: computed(() => props.disabled || sharePending.value),
})

function onPrimaryAction() {
	if (!canTakeAction.value) return
	if (!needsShareReview.value) return handlePrimaryAction()
	previewAction.value = isRunning.value ? 'Restart' : 'Start'
	void shareReview.showPreview(previewAction.value)
}

const stopMenuOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'kill_server',
		label: 'Kill server',
		icon: SlashIcon,
		action: () => initiateAction('Kill'),
	},
])

const messages = defineMessages({
	startBody: {
		id: 'servers.power.share-changes-start-body',
		defaultMessage:
			'Starting the server will push these changes to all invited players. They will be asked to update their instance the next time they play.',
	},
	restartBody: {
		id: 'servers.power.share-changes-restart-body',
		defaultMessage:
			'Restarting the server will push these changes to all invited players. They will be asked to update their instance the next time they play.',
	},
	pushAndStart: {
		id: 'servers.power.push-and-start',
		defaultMessage: 'Push update and start',
	},
	pushAndRestart: {
		id: 'servers.power.push-and-restart',
		defaultMessage: 'Push update and restart',
	},
})
</script>
