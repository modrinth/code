<template>
	<div v-if="error" class="flex w-full flex-col items-center justify-center gap-4 p-4">
		<div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
			<div class="flex flex-col items-center text-center">
				<div class="flex flex-col items-center gap-4">
					<div class="grid place-content-center rounded-full bg-bg-orange p-4">
						<IssuesIcon class="size-12 text-orange" />
					</div>
					<h1 class="m-0 mb-2 w-fit text-4xl font-bold">Failed to load backups</h1>
				</div>
				<p class="text-lg text-secondary">
					We couldn't load your server's backups. Here's what went wrong:
				</p>
				<p>
					<span class="break-all font-mono">{{ error.message }}</span>
				</p>
				<ButtonStyled size="large" color="brand" @click="refetch">
					<button class="mt-6 !w-full">Retry</button>
				</ButtonStyled>
			</div>
		</div>
	</div>
	<div v-else class="contents">
		<BackupCreateModal ref="createBackupModal" />
		<BackupRenameModal ref="renameBackupModal" />
		<BackupRestoreModal ref="restoreBackupModal" />
		<BackupDeleteModal ref="deleteBackupModal" @delete="deleteBackup" />
		<!-- <BackupSettingsModal ref="backupSettingsModal" /> -->

		<div class="mb-6 flex flex-col items-center justify-between gap-4 sm:flex-row">
			<div class="flex flex-col gap-2">
				<div class="flex items-center gap-2">
					<h1 class="m-0 text-2xl font-extrabold text-contrast">Backups</h1>
					<TagItem
						v-tooltip="`${server.backup_quota - server.used_backup_quota} backup slots remaining`"
						class="cursor-help"
						:style="{
							'--_color':
								server.backup_quota <= server.used_backup_quota
									? 'var(--color-red)'
									: server.backup_quota - server.used_backup_quota <= 3
										? 'var(--color-orange)'
										: undefined,
							'--_bg-color':
								server.backup_quota <= server.used_backup_quota
									? 'var(--color-red-bg)'
									: server.backup_quota - server.used_backup_quota <= 3
										? 'var(--color-orange-bg)'
										: undefined,
						}"
					>
						{{ server.used_backup_quota }} / {{ server.backup_quota }}
					</TagItem>
				</div>
				<p class="m-0">
					You can have up to {{ server.backup_quota }} backups at once, stored securely off-site.
				</p>
			</div>
			<div
				class="grid w-full grid-cols-[repeat(auto-fit,_minmax(180px,1fr))] gap-2 sm:flex sm:w-fit sm:flex-row"
			>
				<ButtonStyled type="standard">
					<!-- TODO: When auto backups are implemented re-add the @click event -->
					<button
						v-tooltip="
							'Auto backups are currently unavailable; we apologize for the inconvenience.'
						"
						:disabled="true || server.status === 'installing'"
					>
						<SettingsIcon class="h-5 w-5" />
						Auto backups
					</button>
				</ButtonStyled>
				<ButtonStyled type="standard" color="brand">
					<button
						v-tooltip="backupCreationDisabled"
						class="w-full sm:w-fit"
						:disabled="!!backupCreationDisabled"
						@click="showCreateModel"
					>
						<PlusIcon class="h-5 w-5" />
						Create backup
					</button>
				</ButtonStyled>
			</div>
		</div>

		<div class="flex w-full flex-col gap-2">
			<div
				v-if="backups.length === 0"
				class="mt-6 flex items-center justify-center gap-2 text-center text-secondary"
			>
				<template v-if="server.used_backup_quota">
					<SpinnerIcon class="animate-spin" />
					Loading backups...
				</template>
				<template v-else> You don't have any backups yet. </template>
			</div>
			<BackupItem
				v-for="backup in backups"
				:key="`backup-${backup.id}`"
				:backup="backup"
				:kyros-url="server.node?.instance"
				:jwt="server.node?.token"
				:show-debug-info="showDebugInfo"
				@download="() => triggerDownloadAnimation()"
				@rename="() => renameBackupModal?.show(backup)"
				@restore="() => restoreBackupModal?.show(backup)"
				@lock="
					() => {
						if (backup.locked) {
							unlockBackup(backup.id)
						} else {
							lockBackup(backup.id)
						}
					}
				"
				@delete="
					(skipConfirmation?: boolean) =>
						!skipConfirmation ? deleteBackup(backup) : deleteBackupModal?.show(backup)
				"
				@retry="() => retryBackup(backup.id)"
			/>
		</div>

		<div
			class="over-the-top-download-animation"
			:class="{ 'animation-hidden': !overTheTopDownloadAnimation }"
		>
			<div>
				<div
					class="animation-ring-3 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-40"
				></div>
				<div
					class="animation-ring-2 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-60"
				></div>
				<div
					class="animation-ring-1 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight"
				>
					<DownloadIcon class="h-20 w-20 text-contrast" />
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { DownloadIcon, IssuesIcon, PlusIcon, SettingsIcon, SpinnerIcon } from '@modrinth/assets'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { useStorage } from '@vueuse/core'
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'

import { injectModrinthClient, injectModrinthServerContext, injectNotificationManager } from '../../../providers'

import ButtonStyled from '../../../components/base/ButtonStyled.vue'
import TagItem from '../../../components/base/TagItem.vue'
import BackupCreateModal from '../../../components/servers/backups/BackupCreateModal.vue'
import BackupDeleteModal from '../../../components/servers/backups/BackupDeleteModal.vue'
import BackupItem from '../../../components/servers/backups/BackupItem.vue'
import BackupRenameModal from '../../../components/servers/backups/BackupRenameModal.vue'
import BackupRestoreModal from '../../../components/servers/backups/BackupRestoreModal.vue'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { server } = injectModrinthServerContext()

const props = defineProps<{
	isServerRunning: boolean
	showDebugInfo?: boolean
}>()

const route = useRoute()
const serverId = route.params.id as string

// TODO: pinia
const userPreferences = useStorage(`pyro-server-${serverId}-preferences`, {
	backupWhileRunning: false,
})

defineEmits(['onDownload'])

const backupsQueryKey = ['backups', 'list', serverId]
const {
	data: backupsData,
	error,
	refetch,
} = useQuery({
	queryKey: backupsQueryKey,
	queryFn: () => client.archon.backups_v0.list(serverId),
})

const deleteMutation = useMutation({
	mutationFn: (backupId: string) => client.archon.backups_v0.delete(serverId, backupId),
	onSuccess: () => queryClient.invalidateQueries({ queryKey: backupsQueryKey }),
})

const lockMutation = useMutation({
	mutationFn: (backupId: string) => client.archon.backups_v0.lock(serverId, backupId),
	onSuccess: () => queryClient.invalidateQueries({ queryKey: backupsQueryKey }),
})

const unlockMutation = useMutation({
	mutationFn: (backupId: string) => client.archon.backups_v0.unlock(serverId, backupId),
	onSuccess: () => queryClient.invalidateQueries({ queryKey: backupsQueryKey }),
})

const retryMutation = useMutation({
	mutationFn: (backupId: string) => client.archon.backups_v0.retry(serverId, backupId),
	onSuccess: () => queryClient.invalidateQueries({ queryKey: backupsQueryKey }),
})

const backups = computed(() => {
	if (!backupsData.value) return []
	return [...backupsData.value].sort((a, b) => {
		return new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
	})
})

const overTheTopDownloadAnimation = ref()
const createBackupModal = ref<InstanceType<typeof BackupCreateModal>>()
const renameBackupModal = ref<InstanceType<typeof BackupRenameModal>>()
const restoreBackupModal = ref<InstanceType<typeof BackupRestoreModal>>()
const deleteBackupModal = ref<InstanceType<typeof BackupDeleteModal>>()
// const backupSettingsModal = ref<InstanceType<typeof BackupSettingsModal>>()

const backupCreationDisabled = computed(() => {
	if (props.isServerRunning && !userPreferences.value.backupWhileRunning) {
		return 'Cannot create backup while server is running'
	}
	if (
		server.value.used_backup_quota !== undefined &&
		server.value.backup_quota !== undefined &&
		server.value.used_backup_quota >= server.value.backup_quota
	) {
		return `All ${server.value.backup_quota} of your backup slots are in use`
	}
	if (backups.value.some((backup) => backup.task?.create?.state === 'ongoing')) {
		return 'A backup is already in progress'
	}
	if (server.value.status === 'installing') {
		return 'Cannot create backup while server is installing'
	}
	return undefined
})

const showCreateModel = () => {
	createBackupModal.value?.show()
}

// const showbackupSettingsModal = () => {
// 	backupSettingsModal.value?.show()
// }

function triggerDownloadAnimation() {
	overTheTopDownloadAnimation.value = true
	setTimeout(() => (overTheTopDownloadAnimation.value = false), 500)
}

const lockBackup = (backupId: string) => {
	lockMutation.mutate(backupId, {
		onError: (err) => {
			console.error('Failed to lock backup:', err)
		},
	})
}

const unlockBackup = (backupId: string) => {
	unlockMutation.mutate(backupId, {
		onError: (err) => {
			console.error('Failed to unlock backup:', err)
		},
	})
}

const retryBackup = (backupId: string) => {
	retryMutation.mutate(backupId, {
		onError: (err) => {
			console.error('Failed to retry backup:', err)
		},
	})
}

function deleteBackup(backup?: Archon.Backups.v1.Backup) {
	if (!backup) {
		addNotification({
			type: 'error',
			title: 'Error deleting backup',
			text: 'Backup is null',
		})
		return
	}

	deleteMutation.mutate(backup.id, {
		onError: (err) => {
			const message = err instanceof Error ? err.message : String(err)
			addNotification({
				type: 'error',
				title: 'Error deleting backup',
				text: message,
			})
		},
	})
}
</script>

<style scoped>
.over-the-top-download-animation {
	position: fixed;
	z-index: 100;
	inset: 0;
	display: flex;
	justify-content: center;
	align-items: center;
	pointer-events: none;
	scale: 0.5;
	transition: all 0.5s ease-out;
	opacity: 1;

	&.animation-hidden {
		scale: 0.8;
		opacity: 0;

		.animation-ring-1 {
			width: 25rem;
			height: 25rem;
		}
		.animation-ring-2 {
			width: 50rem;
			height: 50rem;
		}
		.animation-ring-3 {
			width: 100rem;
			height: 100rem;
		}
	}

	> div {
		position: relative;
		display: flex;
		justify-content: center;
		align-items: center;
		width: fit-content;
		height: fit-content;

		> * {
			position: absolute;
			scale: 1;
			transition: all 0.2s ease-out;
			width: 20rem;
			height: 20rem;
		}
	}
}
</style>
