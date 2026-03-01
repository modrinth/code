<template>
	<Transition name="fade" mode="out-in">
		<div
			v-if="error"
			key="error"
			class="flex w-full flex-col items-center justify-center gap-4 p-4"
		>
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

		<div v-else key="content" class="contents">
			<BackupCreateModal ref="createBackupModal" :backups="backupsData ?? []" />
			<BackupRenameModal ref="renameBackupModal" :backups="backupsData ?? []" />
			<BackupRestoreModal ref="restoreBackupModal" />
			<BackupDeleteModal ref="deleteBackupModal" @delete="deleteBackup" />

			<div v-if="backupsData?.length" class="mb-2 flex items-center align-middle justify-between">
				<span class="text-2xl font-semibold text-contrast">Backups</span>
				<ButtonStyled color="brand">
					<button
						v-tooltip="backupCreationDisabled"
						:disabled="!!backupCreationDisabled"
						@click="showCreateModel"
					>
						<PlusIcon class="size-5" />
						Create backup
					</button>
				</ButtonStyled>
			</div>

			<div class="flex w-full flex-col gap-1.5">
				<Transition name="fade" mode="out-in">
					<div
						v-if="groupedBackups.length === 0"
						key="empty"
						class="mt-6 flex flex-col items-center justify-center gap-2 text-center text-secondary"
					>
						<template v-if="!backupsData">
							<SpinnerIcon class="animate-spin" />
							Loading backups...
						</template>
						<template v-else>
							<EmptyState
								type="empty-inbox"
								heading="No backups yet"
								description="Create your first backup"
							>
								<template #actions>
									<ButtonStyled color="brand">
										<button
											v-tooltip="backupCreationDisabled"
											:disabled="!!backupCreationDisabled"
											class="w-min mx-auto"
											@click="showCreateModel"
										>
											<PlusIcon class="size-5" />
											Create backup
										</button>
									</ButtonStyled>
								</template>
							</EmptyState>
						</template>
					</div>

					<div v-else key="list" class="flex flex-col gap-1.5">
						<template v-for="group in groupedBackups" :key="group.label">
							<div class="flex items-center gap-2">
								<component :is="group.icon" v-if="group.icon" class="size-6 text-secondary" />
								<span class="text-lg font-semibold text-secondary">{{ group.label }}</span>
							</div>

							<div class="flex gap-2">
								<div class="flex w-5 justify-center">
									<div class="h-full w-px bg-surface-5" />
								</div>

								<TransitionGroup name="list" tag="div" class="flex flex-1 flex-col gap-3 py-3">
									<BackupItem
										v-for="backup in group.backups"
										:key="`backup-${backup.id}`"
										:backup="backup"
										:restore-disabled="backupRestoreDisabled"
										:kyros-url="server.node?.instance"
										:jwt="server.node?.token"
										:show-debug-info="showDebugInfo"
										@download="() => triggerDownloadAnimation()"
										@rename="() => renameBackupModal?.show(backup)"
										@restore="() => restoreBackupModal?.show(backup)"
										@delete="
											(skipConfirmation?: boolean) =>
												skipConfirmation ? deleteBackup(backup) : deleteBackupModal?.show(backup)
										"
										@retry="() => retryBackup(backup.id)"
									/>
								</TransitionGroup>
							</div>
						</template>
					</div>
				</Transition>
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
	</Transition>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { CalendarIcon, DownloadIcon, IssuesIcon, PlusIcon, SpinnerIcon } from '@modrinth/assets'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import type { Component } from 'vue'
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'

import ButtonStyled from '../../../components/base/ButtonStyled.vue'
import EmptyState from '../../../components/base/EmptyState.vue'
import BackupCreateModal from '../../../components/servers/backups/BackupCreateModal.vue'
import BackupDeleteModal from '../../../components/servers/backups/BackupDeleteModal.vue'
import BackupItem from '../../../components/servers/backups/BackupItem.vue'
import BackupRenameModal from '../../../components/servers/backups/BackupRenameModal.vue'
import BackupRestoreModal from '../../../components/servers/backups/BackupRestoreModal.vue'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '../../../providers'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { server, worldId, backupsState, markBackupCancelled } = injectModrinthServerContext()

const props = defineProps<{
	isServerRunning: boolean
	showDebugInfo?: boolean
}>()

const route = useRoute()
const serverId = route.params.id as string

defineEmits(['onDownload'])

const backupsQueryKey = ['backups', 'list', serverId]
const {
	data: backupsData,
	error,
	refetch,
} = useQuery({
	queryKey: backupsQueryKey,
	queryFn: () => client.archon.backups_v1.list(serverId, worldId.value!),
})

const deleteMutation = useMutation({
	mutationFn: (backupId: string) =>
		client.archon.backups_v1.delete(serverId, worldId.value!, backupId),
	onSuccess: (_data, backupId) => {
		markBackupCancelled(backupId)
		backupsState.delete(backupId)
		queryClient.invalidateQueries({ queryKey: backupsQueryKey })
	},
})

const retryMutation = useMutation({
	mutationFn: (backupId: string) =>
		client.archon.backups_v1.retry(serverId, worldId.value!, backupId),
	onSuccess: () => queryClient.invalidateQueries({ queryKey: backupsQueryKey }),
})

const backups = computed(() => {
	if (!backupsData.value) return []

	const merged = backupsData.value.map((backup) => {
		const progressState = backupsState.get(backup.id)
		if (progressState) {
			const hasOngoingTask = Object.values(progressState).some((task) => task?.state === 'ongoing')
			const hasCompletedTask = Object.values(progressState).some((task) => task?.state === 'done')

			return {
				...backup,
				task: {
					...backup.task,
					...progressState,
				},

				ongoing: hasOngoingTask || (backup.ongoing && !hasCompletedTask),
			}
		}
		return backup
	})

	return merged.sort((a, b) => {
		return new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
	})
})

type BackupGroup = {
	label: string
	icon: Component | null
	backups: Archon.Backups.v1.Backup[]
}

const groupedBackups = computed((): BackupGroup[] => {
	if (!backups.value.length) return []

	const now = dayjs()
	const groups: BackupGroup[] = []

	const addToGroup = (label: string, icon: Component | null, backup: Archon.Backups.v1.Backup) => {
		let group = groups.find((g) => g.label === label)
		if (!group) {
			group = { label, icon, backups: [] }
			groups.push(group)
		}
		group.backups.push(backup)
	}

	for (const backup of backups.value) {
		const created = dayjs(backup.created_at)
		const diffMinutes = now.diff(created, 'minute')
		const isToday = created.isSame(now, 'day')
		const isYesterday = created.isSame(now.subtract(1, 'day'), 'day')
		const diffDays = now.diff(created, 'day')

		if (diffMinutes < 30 && isToday) {
			addToGroup('Just now', CalendarIcon, backup)
		} else if (isToday) {
			addToGroup('Earlier today', CalendarIcon, backup)
		} else if (isYesterday) {
			addToGroup('Yesterday', CalendarIcon, backup)
		} else if (diffDays <= 14) {
			addToGroup('Last 2 weeks', CalendarIcon, backup)
		} else {
			addToGroup('Older', CalendarIcon, backup)
		}
	}

	return groups
})

const overTheTopDownloadAnimation = ref()
const createBackupModal = ref<InstanceType<typeof BackupCreateModal>>()
const renameBackupModal = ref<InstanceType<typeof BackupRenameModal>>()
const restoreBackupModal = ref<InstanceType<typeof BackupRestoreModal>>()
const deleteBackupModal = ref<InstanceType<typeof BackupDeleteModal>>()

const backupRestoreDisabled = computed(() => {
	if (props.isServerRunning) {
		return 'Cannot restore backup while server is running'
	}
	for (const entry of backupsState.values()) {
		if (entry.create?.state === 'ongoing') {
			return 'Cannot restore backup while a backup is being created'
		}
		if (entry.restore?.state === 'ongoing') {
			return 'Cannot restore backup while another restore is in progress'
		}
	}
	return undefined
})

const backupCreationDisabled = computed(() => {
	if (
		server.value.used_backup_quota !== undefined &&
		server.value.backup_quota !== undefined &&
		server.value.used_backup_quota >= server.value.backup_quota
	) {
		return `All ${server.value.backup_quota} of your backup slots are in use`
	}

	for (const entry of backupsState.values()) {
		if (entry.create?.state === 'ongoing') {
			return 'A backup is already in progress'
		}
		if (entry.restore?.state === 'ongoing') {
			return 'Cannot create backup while a restore is in progress'
		}
	}

	// also check API data for ongoing backups (before ws fires)
	if (backupsData.value?.some((backup) => backup.ongoing)) {
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

function triggerDownloadAnimation() {
	overTheTopDownloadAnimation.value = true
	setTimeout(() => (overTheTopDownloadAnimation.value = false), 500)
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
.fade-enter-active,
.fade-leave-active {
	transition:
		opacity 300ms ease-in-out,
		transform 300ms ease-in-out;
}

.fade-enter-from,
.fade-leave-to {
	opacity: 0;
	transform: scale(0.98);
}

.list-enter-active,
.list-leave-active {
	transition: all 200ms ease-in-out;
}

.list-enter-from {
	opacity: 0;
	transform: translateY(-10px);
}

.list-leave-to {
	opacity: 0;
	transform: translateY(10px);
}

.list-move {
	transition: transform 200ms ease-in-out;
}

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
