<template>
	<div v-if="isSiteAdmin" class="flex flex-col gap-6">
		<Teleport to="body">
			<div class="relative z-[100]">
				<ConfirmModal
					ref="clearActionLogModal"
					:title="formatMessage(messages.clearActionLogTitle)"
					:description="formatMessage(messages.clearActionLogConfirmation)"
					:proceed-label="formatMessage(messages.clearActionLogButton)"
					:markdown="false"
					@proceed="clearActionLog"
				/>
				<ConfirmModal
					ref="resetToOnboardingModal"
					:title="formatMessage(messages.resetToOnboardingModalTitle)"
					:description="formatMessage(messages.resetToOnboardingModalDescription)"
					:proceed-label="formatMessage(messages.resetToOnboardingButton)"
					@proceed="confirmResetToOnboarding"
				/>
				<LockServerModal ref="lockServerModal" :server-id="serverId" :server-name="server.name" />
				<ConfirmModal
					ref="resuspendModal"
					:title="formatMessage(messages.resuspendTitle)"
					:description="formatMessage(messages.resuspendDescription, { name: server.name })"
					:proceed-label="formatMessage(messages.resuspendButton)"
					:markdown="false"
					@proceed="resuspend"
				/>
			</div>
		</Teleport>

		<div class="flex flex-col gap-2.5">
			<span class="text-lg font-semibold text-contrast">{{
				formatMessage(messages.actionLogTitle)
			}}</span>
			<div>
				<Button
					v-tooltip="clearActionLogTooltip"
					type="colored"
					color="red"
					:disabled="!canResetServer || isClearingActionLog"
					:loading="isClearingActionLog"
					@click="showClearActionLogModal"
				>
					<TrashIcon class="size-5" />
					{{ formatMessage(messages.clearActionLogButton) }}
				</Button>
			</div>
			<span class="text-primary">{{ formatMessage(messages.clearActionLogDescription) }}</span>
		</div>

		<div class="flex flex-col gap-2.5">
			<span class="text-lg font-semibold text-contrast">
				{{ formatMessage(server.locked_since ? messages.unlockServer : messages.lockServer) }}
			</span>
			<div>
				<Button type="colored" color="red" @click="toggleServerLock">
					<component :is="server.locked_since ? LockOpenIcon : LockIcon" class="size-5" />
					{{ formatMessage(server.locked_since ? messages.unlockServer : messages.lockServer) }}
				</Button>
			</div>
		</div>

		<div v-if="suspensionLift" class="flex flex-col gap-2.5">
			<span class="text-lg font-semibold text-contrast">
				{{ formatMessage(messages.suspensionLiftedTitle) }}
			</span>
			<div class="flex flex-wrap items-center gap-3">
				<Button
					type="colored"
					color="red"
					:disabled="isResuspending"
					:loading="isResuspending"
					@click="resuspendModal?.show()"
				>
					<LockIcon class="size-5" />
					{{ formatMessage(messages.resuspendButton) }}
				</Button>
				<span v-tooltip="formatDateTime(suspensionLift.until)" class="text-primary">
					{{ formatMessage(messages.suspendedAgain, { time: liftEndsIn }) }}
				</span>
			</div>
			<span class="text-secondary">
				{{
					formatMessage(
						suspensionLift.restrict_power
							? messages.powerActionsBlocked
							: messages.powerActionsAllowed,
					)
				}}
			</span>
		</div>

		<div class="flex items-center justify-between gap-4">
			<div class="flex flex-col gap-2.5">
				<span id="drop-client-mods-label" class="text-lg font-semibold text-contrast">
					Drop client mods
				</span>
				<span id="drop-client-mods-description" class="text-primary">
					Remove client-only files when installing modpacks instead of keeping them disabled on the
					server. Applies from the next installation.
				</span>
			</div>
			<Toggle
				id="drop-client-mods"
				:model-value="dropClientMods"
				:disabled="!serverFull || isSettingDropClientMods"
				aria-labelledby="drop-client-mods-label"
				aria-describedby="drop-client-mods-description"
				@update:model-value="(value) => setDropClientMods(!!value)"
			/>
		</div>

		<div class="flex flex-col gap-2.5">
			<span class="text-lg font-semibold text-contrast">
				{{ formatMessage(messages.resetToOnboardingButton) }}
			</span>
			<div>
				<Button
					v-tooltip="supportResetToOnboardingTooltip"
					type="colored"
					color="red"
					:disabled="supportResetToOnboardingDisabled"
					@click="showResetToOnboardingModal"
				>
					<RotateCounterClockwiseIcon class="size-5" />
					{{ formatMessage(messages.resetToOnboardingButton) }}
				</Button>
			</div>
		</div>

		<BackupRepositoryReparation :server-id="serverId" />
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { LockIcon, LockOpenIcon, RotateCounterClockwiseIcon, TrashIcon } from '@modrinth/assets'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { useNow } from '@vueuse/core'
import { computed, ref } from 'vue'

import { Button } from '#ui/components/base/buttons'
import Toggle from '#ui/components/base/Toggle.vue'
import { ConfirmModal } from '#ui/components/modal'
import LockServerModal from '#ui/components/servers/LockServerModal.vue'
import { useFormatDateTime } from '#ui/composables/format-date-time'
import { useRelativeTime } from '#ui/composables/how-ago'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useModrinthServersConsole } from '#ui/composables/server-console'
import { useServerPermissions } from '#ui/composables/server-permissions'
import BackupRepositoryReparation from '#ui/layouts/shared/server-settings/components/backup-repository-reparation.vue'
import { injectServerSettings } from '#ui/layouts/shared/server-settings/providers/server-settings'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

const client = injectModrinthClient()
const { server, serverFull, serverId, worldId, isSiteAdmin } = injectModrinthServerContext()
const serverSettings = injectServerSettings()
const modrinthServersConsole = useModrinthServersConsole()
const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()
const { formatMessage } = useVIntl()
const { canResetServer, permissionDeniedMessage } = useServerPermissions()

const messages = defineMessages({
	actionLogTitle: {
		id: 'servers.settings.support.action-log-title',
		defaultMessage: 'Action log',
	},
	clearActionLogButton: {
		id: 'servers.settings.support.clear-action-log-button',
		defaultMessage: 'Clear action log',
	},
	clearActionLogDescription: {
		id: 'servers.settings.support.clear-action-log-description',
		defaultMessage: 'Delete all action log entries for this server.',
	},
	clearActionLogTitle: {
		id: 'servers.settings.support.clear-action-log-title',
		defaultMessage: 'Clear action log?',
	},
	clearActionLogConfirmation: {
		id: 'servers.settings.support.clear-action-log-confirmation',
		defaultMessage: 'This permanently deletes all action log entries for this server. Continue?',
	},
	clearActionLogSuccess: {
		id: 'servers.settings.support.clear-action-log-success',
		defaultMessage: 'Action log cleared',
	},
	clearActionLogError: {
		id: 'servers.settings.support.clear-action-log-error',
		defaultMessage: 'Failed to clear action log',
	},
	resetToOnboardingButton: {
		id: 'hosting.loader.reset-to-onboarding-button',
		defaultMessage: 'Reset to onboarding',
	},
	resetToOnboardingModalTitle: {
		id: 'hosting.loader.reset-to-onboarding-modal-title',
		defaultMessage: 'Reset to onboarding',
	},
	resetToOnboardingModalDescription: {
		id: 'hosting.loader.reset-to-onboarding-modal-description',
		defaultMessage:
			'This will send the server back into onboarding so setup can be completed again. Are you sure you want to continue?',
	},
	resetToOnboardingSuccessTitle: {
		id: 'hosting.loader.reset-to-onboarding-success-title',
		defaultMessage: 'Server reset to onboarding',
	},
	resetToOnboardingSuccessDescription: {
		id: 'hosting.loader.reset-to-onboarding-success-description',
		defaultMessage: 'The server has been returned to the onboarding flow.',
	},
	failedToResetToOnboarding: {
		id: 'hosting.loader.failed-to-reset-to-onboarding',
		defaultMessage: 'Failed to reset server to onboarding',
	},
	lockServer: {
		id: 'servers.manage.lock-server',
		defaultMessage: 'Lock server',
	},
	unlockServer: {
		id: 'servers.manage.unlock-server',
		defaultMessage: 'Unlock server',
	},
	unlockSuccessTitle: {
		id: 'servers.manage.unlock.success-title',
		defaultMessage: 'Server unlocked',
	},
	unlockSuccessText: {
		id: 'servers.manage.unlock.success-text',
		defaultMessage: '{name} has been unlocked.',
	},
	unlockErrorTitle: {
		id: 'servers.manage.unlock.error-title',
		defaultMessage: 'Failed to unlock server',
	},
	unlockErrorText: {
		id: 'servers.manage.unlock.error-text',
		defaultMessage: 'An error occurred while unlocking this server. Please try again.',
	},
	suspensionLiftedTitle: {
		id: 'servers.settings.support.suspension-lifted.title',
		defaultMessage: 'Suspension lifted',
	},
	suspendedAgain: {
		id: 'servers.settings.support.suspension-lifted.suspended-again',
		defaultMessage: 'Suspended again {time}',
	},
	powerActionsBlocked: {
		id: 'servers.settings.support.suspension-lifted.power-blocked',
		defaultMessage: 'Power actions are blocked while the suspension is lifted.',
	},
	powerActionsAllowed: {
		id: 'servers.settings.support.suspension-lifted.power-allowed',
		defaultMessage: 'Power actions are allowed while the suspension is lifted.',
	},
	resuspendButton: {
		id: 'servers.settings.support.resuspend.button',
		defaultMessage: 'Unlift now',
	},
	resuspendTitle: {
		id: 'servers.settings.support.resuspend.title',
		defaultMessage: 'Unlift suspension now?',
	},
	resuspendDescription: {
		id: 'servers.settings.support.resuspend.description',
		defaultMessage:
			"This ends the temporary lift of {name}'s suspension early. The server will be suspended again right away.",
	},
	resuspendSuccessTitle: {
		id: 'servers.settings.support.resuspend.success-title',
		defaultMessage: 'Server suspended',
	},
	resuspendSuccessText: {
		id: 'servers.settings.support.resuspend.success-text',
		defaultMessage: '{name} has been suspended again.',
	},
	resuspendErrorTitle: {
		id: 'servers.settings.support.resuspend.error-title',
		defaultMessage: 'Failed to unlift the suspension',
	},
})

const lockServerModal = ref<InstanceType<typeof LockServerModal> | null>(null)
const clearActionLogModal = ref<InstanceType<typeof ConfirmModal>>()
const isClearingActionLog = ref(false)
const clearActionLogTooltip = computed(() =>
	canResetServer.value ? undefined : permissionDeniedMessage.value,
)

const resuspendModal = ref<InstanceType<typeof ConfirmModal>>()
const isResuspending = ref(false)
const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })
const now = useNow({ interval: 30_000 })
const suspensionLift = computed(() => {
	const lift = server.value?.suspension_lift
	return lift && new Date(lift.until).getTime() > now.value.getTime() ? lift : null
})
const liftEndsIn = computed(() =>
	suspensionLift.value ? formatRelativeTime(suspensionLift.value.until) : '',
)

const resetToOnboardingModal = ref<InstanceType<typeof ConfirmModal>>()
const isResettingToOnboarding = ref(false)
const supportResetToOnboardingDisabled = computed(
	() => !worldId.value || isResettingToOnboarding.value || !canResetServer.value,
)
const supportResetToOnboardingTooltip = computed(() =>
	!canResetServer.value ? permissionDeniedMessage.value : undefined,
)

const dropClientMods = computed(() => serverFull.value?.drop_client_mods ?? true)

const { mutate: setDropClientMods, isPending: isSettingDropClientMods } = useMutation({
	mutationFn: (value: boolean) =>
		client.archon.servers_internal.setDropClientMods(serverId, { drop_client_mods: value }),
	onSuccess: ({ drop_client_mods }) => {
		queryClient.setQueryData<Archon.Servers.v1.ServerFull>(
			['servers', 'v1', 'detail', serverId],
			(current) => (current ? { ...current, drop_client_mods } : current),
		)
	},
	onError: (error) => {
		console.error(error)
		addNotification({
			type: 'error',
			title: 'Failed to update drop client mods',
		})
	},
})

function showResetToOnboardingModal() {
	if (supportResetToOnboardingDisabled.value) return
	resetToOnboardingModal.value?.show()
}

function showClearActionLogModal() {
	if (!isSiteAdmin.value || !canResetServer.value || isClearingActionLog.value) return
	clearActionLogModal.value?.show()
}

async function clearActionLog() {
	if (!isSiteAdmin.value || !canResetServer.value || isClearingActionLog.value) return

	try {
		isClearingActionLog.value = true
		await client.archon.actions_v1.clear(serverId)
		await queryClient.invalidateQueries({
			queryKey: ['servers', 'action-log', 'v1', 'infinite', serverId],
		})
		addNotification({
			type: 'success',
			title: formatMessage(messages.clearActionLogSuccess),
		})
	} catch (error) {
		console.error(error)
		addNotification({
			type: 'error',
			title: formatMessage(messages.clearActionLogError),
		})
	} finally {
		isClearingActionLog.value = false
	}
}

async function toggleServerLock() {
	if (!isSiteAdmin.value || !server.value) return

	if (!server.value.locked_since) {
		lockServerModal.value?.show()
		return
	}

	const name = server.value.name
	try {
		await client.archon.servers_internal.unlock(serverId)
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
			queryClient.invalidateQueries({ queryKey: ['servers', 'locks'] }),
		])
		addNotification({
			type: 'success',
			title: formatMessage(messages.unlockSuccessTitle),
			text: formatMessage(messages.unlockSuccessText, { name }),
		})
	} catch {
		addNotification({
			type: 'error',
			title: formatMessage(messages.unlockErrorTitle),
			text: formatMessage(messages.unlockErrorText),
		})
	}
}

async function resuspend() {
	if (!isSiteAdmin.value || isResuspending.value) return

	const name = server.value.name
	try {
		isResuspending.value = true
		await client.archon.servers_internal.resuspend(serverId)
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
			queryClient.invalidateQueries({
				queryKey: ['servers', 'action-log', 'v1', 'infinite', serverId],
			}),
		])
		addNotification({
			type: 'success',
			title: formatMessage(messages.resuspendSuccessTitle),
			text: formatMessage(messages.resuspendSuccessText, { name }),
		})
		serverSettings.closeModal?.()
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.resuspendErrorTitle),
			text: error instanceof Error ? error.message : undefined,
		})
	} finally {
		isResuspending.value = false
	}
}

async function confirmResetToOnboarding() {
	if (supportResetToOnboardingDisabled.value || !worldId.value) return

	try {
		isResettingToOnboarding.value = true
		await client.archon.servers_v1.resetToOnboarding(serverId, worldId.value)
		modrinthServersConsole.clear()
		try {
			await client.kyros.logs_v1.clear()
		} catch (error) {
			console.error('Failed to clear server logs:', error)
		}
		queryClient.setQueryData<Archon.Servers.v0.Server>(
			['servers', 'detail', serverId],
			(current) => (current ? { ...current, flows: { ...current.flows, intro: true } } : current),
		)
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
			queryClient.invalidateQueries({ queryKey: ['servers', 'v1', 'detail', serverId] }),
		])
		addNotification({
			type: 'success',
			title: formatMessage(messages.resetToOnboardingSuccessTitle),
			text: formatMessage(messages.resetToOnboardingSuccessDescription),
		})
		serverSettings.closeModal?.()
	} catch (err) {
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : formatMessage(messages.failedToResetToOnboarding),
		})
	} finally {
		isResettingToOnboarding.value = false
	}
}
</script>
