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
							<div class="mx-auto flex flex-col justify-center p-6 text-center">
								<div data-svg-wrapper>
									<svg
										viewBox="0 0 250 200"
										fill="none"
										class="h-[200px] w-[250px]"
										xmlns="http://www.w3.org/2000/svg"
									>
										<path
											fill-rule="evenodd"
											clip-rule="evenodd"
											d="M63 134H154C154.515 134 155.017 133.944 155.5 133.839C155.983 133.944 156.485 134 157 134H209C212.866 134 216 130.866 216 127C216 123.134 212.866 120 209 120H203C199.134 120 196 116.866 196 113C196 109.134 199.134 106 203 106H222C225.866 106 229 102.866 229 99C229 95.134 225.866 92 222 92H200C203.866 92 207 88.866 207 85C207 81.134 203.866 78 200 78H136C139.866 78 143 74.866 143 71C143 67.134 139.866 64 136 64H79C75.134 64 72 67.134 72 71C72 74.866 75.134 78 79 78H39C35.134 78 32 81.134 32 85C32 88.866 35.134 92 39 92H64C67.866 92 71 95.134 71 99C71 102.866 67.866 106 64 106H24C20.134 106 17 109.134 17 113C17 116.866 20.134 120 24 120H63C59.134 120 56 123.134 56 127C56 130.866 59.134 134 63 134ZM226 134C229.866 134 233 130.866 233 127C233 123.134 229.866 120 226 120C222.134 120 219 123.134 219 127C219 130.866 222.134 134 226 134Z"
											fill="var(--surface-2, #1D1F23)"
										/>
										<path
											fill-rule="evenodd"
											clip-rule="evenodd"
											d="M113.119 112.307C113.04 112.86 113 113.425 113 114C113 120.627 118.373 126 125 126C131.627 126 137 120.627 137 114C137 113.425 136.96 112.86 136.881 112.307H166V139C166 140.657 164.657 142 163 142H87C85.3431 142 84 140.657 84 139V112.307H113.119Z"
											fill="var(--surface-1, #16181C)"
										/>
										<path
											fill-rule="evenodd"
											clip-rule="evenodd"
											d="M138 112C138 119.18 132.18 125 125 125C117.82 125 112 119.18 112 112C112 111.767 112.006 111.536 112.018 111.307H84L93.5604 83.0389C93.9726 81.8202 95.1159 81 96.4023 81H153.598C154.884 81 156.027 81.8202 156.44 83.0389L166 111.307H137.982C137.994 111.536 138 111.767 138 112Z"
											fill="var(--surface-1, #16181C)"
										/>
										<path
											fill-rule="evenodd"
											clip-rule="evenodd"
											d="M136.098 112.955C136.098 118.502 131.129 124 125 124C118.871 124 113.902 118.502 113.902 112.955C113.902 112.775 113.908 111.596 113.918 111.419H93L101.161 91.5755C101.513 90.6338 102.489 90 103.587 90H146.413C147.511 90 148.487 90.6338 148.839 91.5755L157 111.419H136.082C136.092 111.596 136.098 112.775 136.098 112.955Z"
											fill="var(--surface-2, #1D1F23)"
										/>
										<path
											fill-rule="evenodd"
											clip-rule="evenodd"
											d="M85.25 111.512V138C85.25 138.966 86.0335 139.75 87 139.75H163C163.966 139.75 164.75 138.966 164.75 138V111.512L155.255 83.4393C155.015 82.7285 154.348 82.25 153.598 82.25H96.4023C95.6519 82.25 94.985 82.7285 94.7446 83.4393L85.25 111.512Z"
											stroke="var(--surface-4, #34363C)"
											stroke-width="2.5"
										/>
										<path
											d="M98 111C101.937 111 106.185 111 110.745 111C112.621 111 112.621 112.319 112.621 113C112.621 119.627 118.117 125 124.897 125C131.677 125 137.173 119.627 137.173 113C137.173 112.319 137.173 111 139.05 111H164M90.5737 111H93H90.5737Z"
											stroke="var(--surface-4, #34363C)"
											stroke-width="2.5"
											stroke-linecap="round"
											stroke-linejoin="round"
										/>
										<path
											d="M150.1 58.3027L139 70.7559M124.1 54V70.7559V54ZM98 58.3027L109.1 70.7559L98 58.3027Z"
											stroke="var(--surface-3, #27292E)"
											stroke-width="2.5"
											stroke-linecap="round"
											stroke-linejoin="round"
										/>
									</svg>
								</div>
								<div class="flex flex-col gap-4 -mt-4">
									<div class="flex flex-col gap-1.5">
										<span class="text-lg text-contrast md:text-2xl">No backups yet</span>
										<span class="max-w-[256px] text-sm md:text-base leading-6 text-secondary">
											Create your first backup
										</span>
									</div>
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
								</div>
							</div>
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
const { server, backupsState, markBackupCancelled } = injectModrinthServerContext()

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
	queryFn: () => client.archon.backups_v0.list(serverId),
})

const deleteMutation = useMutation({
	mutationFn: (backupId: string) => client.archon.backups_v0.delete(serverId, backupId),
	onSuccess: (_data, backupId) => {
		markBackupCancelled(backupId)
		backupsState.delete(backupId)
		queryClient.invalidateQueries({ queryKey: backupsQueryKey })
	},
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
// const backupSettingsModal = ref<InstanceType<typeof BackupSettingsModal>>()

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
