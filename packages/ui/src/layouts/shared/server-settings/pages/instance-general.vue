<template>
	<div class="flex h-full w-full flex-col gap-6">
		<Teleport to="body">
			<div class="relative z-[100]">
				<ConfirmModal
					ref="resetWorldFilesModal"
					:title="formatMessage(messages.resetWorldFilesModalTitle)"
					:description="formatMessage(messages.resetWorldFilesModalDescription)"
					:proceed-label="formatMessage(messages.resetWorldFilesButton)"
					@proceed="resetWorldFiles"
				/>
				<ConfirmModal
					ref="deleteInstanceModal"
					:title="formatMessage(messages.deleteInstanceModalTitle)"
					:description="formatMessage(messages.deleteInstanceModalDescription)"
					:proceed-label="formatMessage(messages.deleteInstanceButton)"
					@proceed="deleteInstance"
				/>
				<ServerSetupModal
					ref="setupModal"
					@reinstall="onResetEverything"
					@browse-modpacks="onBrowseModpacks"
				/>
			</div>
		</Teleport>

		<div class="flex max-w-[496px] flex-col gap-2.5">
			<label for="instance-name-field" class="font-semibold text-contrast">
				{{ formatMessage(messages.instanceNameLabel) }}
			</label>
			<StyledInput
				id="instance-name-field"
				v-model="editingInstanceName"
				v-tooltip="nameSaveDisabledTooltip"
				:disabled="isNameSaveDisabled"
				:maxlength="64"
			/>
			<span class="text-primary">{{ formatMessage(messages.instanceNameDescription) }}</span>
		</div>

		<div class="flex flex-col gap-2.5">
			<span class="font-semibold text-contrast">
				{{ formatMessage(messages.dangerZoneLabel) }}
			</span>
			<div class="flex flex-col gap-4 rounded-2xl border border-solid border-surface-5 p-4">
				<div class="flex flex-col items-start gap-2.5">
					<ButtonStyled color="red">
						<button
							v-tooltip="resetWorldFilesDisabledTooltip"
							class="!shadow-none"
							:disabled="isResetWorldFilesDisabled"
							@click="resetWorldFilesModal?.show()"
						>
							<RotateCounterClockwiseIcon class="size-5" />
							{{ formatMessage(messages.resetWorldFilesButton) }}
						</button>
					</ButtonStyled>
					<span class="text-primary">
						{{ formatMessage(messages.resetWorldFilesDescription) }}
					</span>
				</div>

				<div class="flex flex-col items-start gap-2.5">
					<ButtonStyled color="red">
						<button class="!shadow-none" :disabled="isResetDisabled" @click="setupModal?.show()">
							<TrashIcon class="size-5" />
							{{ formatMessage(messages.resetEverythingButton) }}
						</button>
					</ButtonStyled>
					<span class="text-primary">
						{{ formatMessage(messages.resetEverythingDescription) }}
					</span>
				</div>

				<div class="flex flex-col items-start gap-2.5">
					<ButtonStyled color="red">
						<button
							v-tooltip="deleteInstanceDisabledTooltip"
							class="!shadow-none"
							:disabled="isDeleteInstanceDisabled"
							@click="showDeleteInstanceModal"
						>
							<TrashIcon class="size-5" />
							{{ formatMessage(messages.deleteInstanceButton) }}
						</button>
					</ButtonStyled>
					<span class="text-primary">
						{{ formatMessage(messages.deleteInstanceDescription) }}
					</span>
				</div>
			</div>
		</div>

		<SaveBanner
			:is-visible="
				(canUseAdvancedSettings && hasUnsavedName && isInstanceNameValid) || isUpdatingName
			"
			:server-id="serverId"
			:is-updating="isUpdatingName"
			:save="saveInstanceName"
			:reset="resetInstanceName"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { RotateCounterClockwiseIcon, TrashIcon } from '@modrinth/assets'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { ButtonStyled, ConfirmModal, StyledInput } from '#ui/components'
import SaveBanner from '#ui/components/servers/SaveBanner.vue'
import ServerSetupModal from '#ui/components/servers/ServerSetupModal.vue'
import { useModrinthServersConsole } from '#ui/composables'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerPermissions } from '#ui/composables/server-permissions'
import { injectServerSettings } from '#ui/layouts/shared/server-settings'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const { server, serverId, worldId, isSyncingContent, busyReasons } = injectModrinthServerContext()
const { addNotification } = injectNotificationManager()
const serverSettings = injectServerSettings()
const queryClient = useQueryClient()
const modrinthServersConsole = useModrinthServersConsole()
const { canUseAdvancedSettings, canResetServer, permissionDeniedMessage } = useServerPermissions()
const route = useRoute()
const router = useRouter()

const setupModal = ref<InstanceType<typeof ServerSetupModal>>()
const resetWorldFilesModal = ref<InstanceType<typeof ConfirmModal>>()
const deleteInstanceModal = ref<InstanceType<typeof ConfirmModal>>()
const editingInstanceName = ref('')

const messages = defineMessages({
	instanceNameLabel: {
		id: 'server.instance-settings.general.instance-name.label',
		defaultMessage: 'Instance name',
	},
	instanceNameDescription: {
		id: 'server.instance-settings.general.instance-name.description',
		defaultMessage: 'This name is only visible on Modrinth.',
	},
	instanceNameSavedTitle: {
		id: 'server.instance-settings.general.instance-name.saved.title',
		defaultMessage: 'Instance renamed',
	},
	instanceNameSavedDescription: {
		id: 'server.instance-settings.general.instance-name.saved.description',
		defaultMessage: 'The instance name has been updated.',
	},
	instanceNameSaveError: {
		id: 'server.instance-settings.general.instance-name.error',
		defaultMessage: 'Failed to update instance name',
	},
	instanceFallbackName: {
		id: 'server.instance-settings.general.instance-name.fallback',
		defaultMessage: 'Instance',
	},
	dangerZoneLabel: {
		id: 'server.instance-settings.general.danger-zone.label',
		defaultMessage: 'Danger zone',
	},
	resetWorldFilesButton: {
		id: 'server.instance-settings.general.reset-world-files.button',
		defaultMessage: 'Reset world data',
	},
	resetWorldFilesDescription: {
		id: 'server.instance-settings.general.reset-world-files.description',
		defaultMessage:
			'Delete the current world data and generate a new one. Your content and files will stay the same.',
	},
	resetWorldFilesModalTitle: {
		id: 'server.instance-settings.general.reset-world-files.modal.title',
		defaultMessage: 'Reset world data',
	},
	resetWorldFilesModalDescription: {
		id: 'server.instance-settings.general.reset-world-files.modal.description',
		defaultMessage:
			'This deletes the world directory for this instance. Installed content and settings will stay the same.',
	},
	resetWorldFilesStartedTitle: {
		id: 'server.instance-settings.general.reset-world-files.started.title',
		defaultMessage: 'World data reset started',
	},
	resetWorldFilesStartedDescription: {
		id: 'server.instance-settings.general.reset-world-files.started.description',
		defaultMessage: 'The world directory is being reset.',
	},
	resetWorldFilesError: {
		id: 'server.instance-settings.general.reset-world-files.error',
		defaultMessage: 'Failed to reset world data',
	},
	resetEverythingButton: {
		id: 'server.instance-settings.general.reset-everything.button',
		defaultMessage: 'Reset everything',
	},
	resetEverythingDescription: {
		id: 'server.instance-settings.general.reset-everything.description',
		defaultMessage:
			'Reset your instance completely. This removes world data, content, and any configuration. A backup of the previous instance will remain available.',
	},
	deleteInstanceButton: {
		id: 'server.instance-settings.general.delete-instance.button',
		defaultMessage: 'Delete instance',
	},
	deleteInstanceDescription: {
		id: 'server.instance-settings.general.delete-instance.description',
		defaultMessage:
			'Permanently delete this instance, including its content, files, settings, and backups.',
	},
	deleteInstanceModalTitle: {
		id: 'server.instance-settings.general.delete-instance.modal.title',
		defaultMessage: 'Delete instance',
	},
	deleteInstanceModalDescription: {
		id: 'server.instance-settings.general.delete-instance.modal.description',
		defaultMessage:
			'This deletes the instance, including its content, files, settings, and backups. This cannot be undone.',
	},
	deleteInstanceSuccessTitle: {
		id: 'server.instance-settings.general.delete-instance.success.title',
		defaultMessage: 'Instance deleted',
	},
	deleteInstanceSuccessDescription: {
		id: 'server.instance-settings.general.delete-instance.success.description',
		defaultMessage: 'The instance has been deleted.',
	},
	deleteInstanceError: {
		id: 'server.instance-settings.general.delete-instance.error',
		defaultMessage: 'Failed to delete instance',
	},
})

type WorldSummaryCacheItem = {
	id: string
}

type DeleteInstanceOptimisticState = {
	serverFull: Archon.Servers.v1.ServerFull | undefined
	worldSummary: WorldSummaryCacheItem[] | undefined
}

const serverFullQuery = useQuery({
	queryKey: ['servers', 'v1', 'detail', serverId],
	queryFn: () => client.archon.servers_v1.get(serverId),
	staleTime: 30_000,
})

const currentWorld = computed(() => {
	const id = worldId.value
	if (!id) return null
	return serverFullQuery.data.value?.worlds.find((world) => world.id === id) ?? null
})

const instanceName = computed(
	() => currentWorld.value?.name ?? formatMessage(messages.instanceFallbackName),
)
const trimmedInstanceName = computed(() => editingInstanceName.value.trim())
const isResetDisabled = computed(
	() =>
		!worldId.value ||
		server.value?.status === 'installing' ||
		isSyncingContent.value ||
		busyReasons.value.length > 0,
)

const patchWorldMutation = useMutation({
	mutationFn: (name: string) =>
		client.archon.servers_v1.patchWorld(serverId, worldId.value!, { name }),
	onSuccess: async () => {
		await invalidateServerState()
		addNotification({
			type: 'success',
			title: formatMessage(messages.instanceNameSavedTitle),
			text: formatMessage(messages.instanceNameSavedDescription),
		})
	},
	onError: (error) => {
		addNotification({
			type: 'error',
			text: error instanceof Error ? error.message : formatMessage(messages.instanceNameSaveError),
		})
	},
})

const resetWorldMutation = useMutation({
	mutationFn: () => client.archon.content_v1.resetWorld(serverId, worldId.value!),
	onSuccess: async () => {
		await invalidateServerState()
		addNotification({
			type: 'success',
			title: formatMessage(messages.resetWorldFilesStartedTitle),
			text: formatMessage(messages.resetWorldFilesStartedDescription),
		})
	},
	onError: (error) => {
		addNotification({
			type: 'error',
			text: error instanceof Error ? error.message : formatMessage(messages.resetWorldFilesError),
		})
	},
})

const deleteWorldMutation = useMutation({
	mutationFn: (deletedWorldId: string) =>
		client.archon.servers_v1.deleteWorld(serverId, deletedWorldId),
	onMutate: async (deletedWorldId): Promise<DeleteInstanceOptimisticState> => {
		const serverFullKey = ['servers', 'v1', 'detail', serverId] as const
		const worldSummaryKey = ['servers', 'worlds', 'summary', 'v1', serverId] as const

		await Promise.all([
			queryClient.cancelQueries({ queryKey: serverFullKey }),
			queryClient.cancelQueries({ queryKey: worldSummaryKey }),
		])

		const previousServerFull = queryClient.getQueryData<Archon.Servers.v1.ServerFull>(serverFullKey)
		const previousWorldSummary = queryClient.getQueryData<WorldSummaryCacheItem[]>(worldSummaryKey)

		queryClient.setQueryData<Archon.Servers.v1.ServerFull | undefined>(serverFullKey, (previous) =>
			previous
				? {
						...previous,
						worlds: previous.worlds.filter((world) => world.id !== deletedWorldId),
					}
				: previous,
		)
		queryClient.setQueryData<WorldSummaryCacheItem[] | undefined>(worldSummaryKey, (previous) =>
			previous?.filter((world) => world.id !== deletedWorldId),
		)
		queryClient.removeQueries({ queryKey: ['content', 'list', 'v1', serverId, deletedWorldId] })
		queryClient.removeQueries({
			queryKey: ['servers', 'properties', 'v1', serverId, deletedWorldId],
		})
		queryClient.removeQueries({
			queryKey: ['servers', 'startup', 'v1', serverId, deletedWorldId],
		})

		serverSettings.closeModal?.()
		if (route.path !== instancesPath.value) {
			void router.push(instancesPath.value)
		}

		return {
			serverFull: previousServerFull,
			worldSummary: previousWorldSummary,
		}
	},
	onSuccess: () => {
		addNotification({
			type: 'success',
			title: formatMessage(messages.deleteInstanceSuccessTitle),
			text: formatMessage(messages.deleteInstanceSuccessDescription),
		})
	},
	onError: (error, _deletedWorldId, context) => {
		if (context?.serverFull) {
			queryClient.setQueryData(['servers', 'v1', 'detail', serverId], context.serverFull)
		}

		if (context?.worldSummary) {
			queryClient.setQueryData(
				['servers', 'worlds', 'summary', 'v1', serverId],
				context.worldSummary,
			)
		}

		addNotification({
			type: 'error',
			text: error instanceof Error ? error.message : formatMessage(messages.deleteInstanceError),
		})
	},
	onSettled: (_data, _error, deletedWorldId) => {
		void invalidateServerState(deletedWorldId)
	},
})

const isUpdatingName = computed(() => patchWorldMutation.isPending.value)
const isNameSaveDisabled = computed(
	() => !worldId.value || isUpdatingName.value || !canUseAdvancedSettings.value,
)
const nameSaveDisabledTooltip = computed(() =>
	canUseAdvancedSettings.value ? undefined : permissionDeniedMessage.value,
)
const isInstanceNameValid = computed(() => trimmedInstanceName.value.length > 0)
const hasUnsavedName = computed(() => trimmedInstanceName.value !== instanceName.value)
const isResetWorldFilesDisabled = computed(
	() => isResetDisabled.value || resetWorldMutation.isPending.value || !canResetServer.value,
)
const resetWorldFilesDisabledTooltip = computed(() =>
	canResetServer.value ? undefined : permissionDeniedMessage.value,
)
const isDeleteInstanceDisabled = computed(
	() => isResetDisabled.value || deleteWorldMutation.isPending.value || !canResetServer.value,
)
const deleteInstanceDisabledTooltip = computed(() =>
	canResetServer.value ? undefined : permissionDeniedMessage.value,
)
const instancesPath = computed(() => `/hosting/manage/${encodeURIComponent(serverId)}/instances`)

watch(
	instanceName,
	(name) => {
		editingInstanceName.value = name
	},
	{ immediate: true },
)

function onResetEverything() {
	modrinthServersConsole.clear()
	queryClient.removeQueries({ queryKey: ['servers', 'ws-state', serverId] })
	void invalidateServerState()
	serverSettings.closeModal?.()
}

async function saveInstanceName() {
	if (isNameSaveDisabled.value || !isInstanceNameValid.value || !hasUnsavedName.value) return
	await patchWorldMutation.mutateAsync(trimmedInstanceName.value)
}

function resetInstanceName() {
	editingInstanceName.value = instanceName.value
}

function resetWorldFiles() {
	if (isResetWorldFilesDisabled.value) return
	resetWorldMutation.mutate()
}

function showDeleteInstanceModal() {
	if (isDeleteInstanceDisabled.value) return
	deleteInstanceModal.value?.show()
}

function deleteInstance() {
	const deletedWorldId = worldId.value
	if (isDeleteInstanceDisabled.value || !deletedWorldId) return

	deleteWorldMutation.mutate(deletedWorldId)
}

function invalidateServerState(targetWorldId = worldId.value) {
	return Promise.all([
		queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
		queryClient.invalidateQueries({ queryKey: ['servers', 'v1', 'detail', serverId] }),
		queryClient.invalidateQueries({
			queryKey: ['servers', 'worlds', 'summary', 'v1', serverId],
		}),
		queryClient.invalidateQueries({ queryKey: ['files', serverId] }),
		targetWorldId
			? queryClient.invalidateQueries({
					queryKey: ['content', 'list', 'v1', serverId, targetWorldId],
				})
			: Promise.resolve(),
		targetWorldId
			? queryClient.invalidateQueries({
					queryKey: ['servers', 'properties', 'v1', serverId, targetWorldId],
				})
			: Promise.resolve(),
		targetWorldId
			? queryClient.invalidateQueries({
					queryKey: ['servers', 'startup', 'v1', serverId, targetWorldId],
				})
			: Promise.resolve(),
	])
}

function onBrowseModpacks() {
	serverSettings.browseModpacks({
		serverId,
		worldId: worldId.value,
		from: 'reset-server',
	})
}
</script>
