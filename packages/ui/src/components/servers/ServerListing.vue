<template>
	<div>
		<NuxtLink :to="isDisabled ? '' : `/hosting/manage/${server_id}`">
			<div
				class="flex flex-row items-center overflow-x-hidden rounded-2xl border-[1px] border-solid border-surface-5 bg-bg-raised p-4 transition-transform duration-100"
				:class="{
					'!rounded-b-none border-b-0': hasNotice,
					'bg-surface-2': isDisabled,
					'active:scale-95': !isDisabled && !hasNotice,
				}"
				data-pyro-server-listing
				:data-pyro-server-listing-id="server_id"
			>
				<div
					v-if="hasIconOverlay"
					class="flex size-16 items-center justify-center rounded-xl border-[1px] border-solid border-button-border bg-button-bg shadow-sm"
				>
					<ServerIcon :image="image ?? undefined" :disabled="isDisabled" />
					<SpinnerIcon
						v-if="isProvisioning || isUpgrading"
						class="size-8 animate-spin absolute text-contrast"
					/>
					<LockIcon v-else class="size-8 absolute" />
				</div>
				<ServerIcon v-else :image="image ?? undefined" :disabled="isDisabled" />
				<div class="ml-4 flex flex-col gap-2.5">
					<div class="flex flex-row items-center gap-2">
						<h2
							class="m-0 text-xl font-bold"
							:class="{ 'text-contrast': !isDisabled, 'text-primary': isDisabled }"
						>
							{{ name }}
						</h2>
						<div
							v-if="isConfiguring"
							class="flex min-w-0 items-center gap-2 truncate text-sm font-medium text-brand rounded-full bg-brand-highlight border border-solid border-brand px-2.5 h-[28px]"
						>
							<SparklesIcon class="size-5 shrink-0 font-semibold" /> New
						</div>
					</div>

					<div
						v-if="projectData?.title"
						class="m-0 flex flex-row items-center gap-2 text-sm font-medium"
						:class="{ 'text-secondary': isDisabled }"
					>
						<Avatar
							:src="iconUrl"
							no-shadow
							style="min-height: 20px; min-width: 20px; height: 20px; width: 20px"
							alt="Server Icon"
						/>
						Using {{ projectData?.title || 'Unknown' }}
					</div>

					<ServerInfoLabels
						:server-data="
							isConfiguring
								? { net }
								: {
										game,
										mc_version,
										loader,
										loader_version,
										net,
										online,
										players: playerCount
											? { current: playerCount.current, max: playerCount.max }
											: undefined,
									}
						"
						:show-game-label="showGameLabel"
						:show-loader-label="showLoaderLabel"
						:show-player-count="showPlayerCount"
						:class="{ 'text-secondary': isDisabled }"
						:linked="false"
						class="pointer-events-none flex w-full flex-row flex-wrap items-center gap-2 text-primary *:hidden sm:flex-row sm:*:flex"
					/>
				</div>
			</div>
		</NuxtLink>

		<div v-if="noticeType" class="server-listing-notice">
			<div v-if="noticeType === 'provisioning'" class="flex gap-2">
				Please wait while we set up your server. This should only take a minute.
			</div>
			<div v-else-if="noticeType === 'upgrading'" class="flex gap-2">
				Your server's hardware is currently being upgraded and will be back online shortly.
			</div>
			<div v-else-if="noticeType === 'cancelled' || noticeType === 'paymentfailed'">
				Your subscription was cancelled<template v-if="cancellationDate">
					on
					<span class="font-medium text-contrast">
						{{ formatDate(cancellationDate) }}
					</span></template
				><template v-if="noticeType === 'paymentfailed'"> due to payment failure</template
				>.<template v-if="!isFilesExpired">
					Your files will be kept for
					<span class="font-medium text-red">30 days</span> and can be downloaded below before
					they're deleted.</template
				>
			</div>
			<div v-else-if="noticeType === 'setToCancel'">
				Your subscription is set to cancel<template v-if="cancellationDate">
					on
					<span class="font-medium text-contrast">
						{{ formatDate(cancellationDate) }}
					</span></template
				>.<template v-if="!isFilesExpired">
					Your files will be preserved for 30 days after cancellation.
				</template>
			</div>
			<div v-else-if="noticeType === 'moderated'">
				Your server has been suspended by moderation action.
			</div>
			<div v-else>
				Your server has been suspended. Please contact Modrinth Support for more information.
			</div>

			<div v-if="noticeButtons" class="flex gap-2">
				<ButtonStyled
					v-if="noticeButtons.downloadBackup && onDownloadBackup"
					type="outlined"
					circular
				>
					<button
						v-tooltip="'Download latest backup'"
						class="!border-surface-5"
						@click="onDownloadBackup"
					>
						<DownloadIcon />
					</button>
				</ButtonStyled>
				<ButtonStyled v-if="noticeButtons.copyId" type="outlined">
					<button
						v-tooltip="'Copy code to clipboard'"
						class="!border-surface-5"
						@click="copyToClipboard(server_id)"
					>
						<template v-if="copied"> Copied <CheckIcon class="text-green" /> </template>
						<template v-else> Copy ID <CopyIcon /> </template>
					</button>
				</ButtonStyled>
				<ButtonStyled v-if="noticeButtons.support">
					<a href="https://support.modrinth.com/en/" target="_blank"
						><MessagesSquareIcon /> Support
					</a>
				</ButtonStyled>
				<ButtonStyled v-if="noticeButtons.manageBilling" color="brand">
					<AutoLink :to="`/settings/billing#server-${server_id}`">
						<CardIcon /> Manage billing
					</AutoLink>
				</ButtonStyled>
				<ButtonStyled v-if="noticeButtons.resubscribe && onResubscribe" color="brand">
					<button @click="onResubscribe"><RotateCounterClockwiseIcon /> Resubscribe</button>
				</ButtonStyled>
			</div>
		</div>

		<div v-if="pendingChange && status !== 'suspended'" class="server-listing-notice">
			<div>
				Your server will {{ pendingChange.verb.toLowerCase() }} to the "{{
					pendingChange.planSize
				}}" plan on
				<span class="font-medium text-contrast">{{ formatDate(pendingChange.date) }}</span
				>.
			</div>
			<ServersSpecs
				class="!font-normal !text-primary"
				:ram="Math.round((pendingChange.ramGb ?? 0) * 1024)"
				:storage="Math.round((pendingChange.storageGb ?? 0) * 1024)"
				:cpus="pendingChange.cpuBurst"
				bursting-link="https://docs.modrinth.com/servers/bursting"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import {
	DownloadIcon,
	LockIcon,
	MessagesSquareIcon,
	SparklesIcon,
	SpinnerIcon,
} from '@modrinth/assets'
import { AutoLink, ButtonStyled } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import {
	CardIcon,
	CheckIcon,
	CopyIcon,
	RotateCounterClockwiseIcon,
} from '../../../../assets/generated-icons'
import { useFormatDateTime } from '../../composables'
import { injectModrinthClient } from '../../providers/api-client'
import Avatar from '../base/Avatar.vue'
import ServersSpecs from '../billing/ServersSpecs.vue'
import ServerIcon from './icons/ServerIcon.vue'
import ServerInfoLabels from './labels/ServerInfoLabels.vue'

const formatDate = useFormatDateTime({ dateStyle: 'long' })

export type PendingChange = {
	planSize: string
	cpu: number
	cpuBurst: number
	ramGb: number
	swapGb?: number
	storageGb?: number
	date: string | number | Date
	intervalChange?: string | null
	verb: string
}

type ServerListingProps = {
	server_id: string
	name: string
	status: Archon.Servers.v0.Status
	suspension_reason?: Archon.Servers.v0.SuspensionReason | null
	game?: Archon.Servers.v0.Game
	mc_version?: string | null
	loader?: Archon.Servers.v0.Loader | null
	loader_version?: string | null
	net?: Archon.Servers.v0.Net
	upstream?: Archon.Servers.v0.Upstream | null
	flows?: Archon.Servers.v0.Flows
	pendingChange?: PendingChange
	online?: boolean
	playerCount?: {
		current?: number
		max?: number
	}
	cancellationDate?: string | Date | null
	onResubscribe?: (() => void) | null
	onDownloadBackup?: (() => void) | null
}

const props = defineProps<ServerListingProps>()

const { kyros, labrinth } = injectModrinthClient()

const isConfiguring = computed(() => props.flows?.intro)
const isProvisioning = computed(() => props.status === 'installing' && !isConfiguring.value)
const isUpgrading = computed(
	() => props.status === 'suspended' && props.suspension_reason === 'upgrading',
)
const isDisabled = computed(() => props.status === 'suspended' || isProvisioning.value)
const isSetToCancel = computed(() => !!props.cancellationDate && props.status !== 'suspended')
const isFilesExpired = computed(() => {
	if (!props.cancellationDate) return false
	const cancellation = new Date(props.cancellationDate)
	const thirtyDaysLater = new Date(cancellation.getTime() + 30 * 24 * 60 * 60 * 1000)
	return new Date() > thirtyDaysLater
})

const hasIconOverlay = computed(
	() => isProvisioning.value || isUpgrading.value || props.status === 'suspended',
)

type NoticeType =
	| 'provisioning'
	| 'upgrading'
	| 'cancelled'
	| 'paymentfailed'
	| 'moderated'
	| 'suspended'
	| 'setToCancel'

const noticeType = computed<NoticeType | null>(() => {
	if (isProvisioning.value) return 'provisioning'
	if (props.status === 'suspended') {
		switch (props.suspension_reason) {
			case 'upgrading':
				return 'upgrading'
			case 'cancelled':
				return 'cancelled'
			case 'paymentfailed':
				return 'paymentfailed'
			case 'moderated':
				return 'moderated'
			default:
				return 'suspended'
		}
	}
	if (isSetToCancel.value) return 'setToCancel'
	return null
})

type NoticeButtons = {
	downloadBackup?: boolean
	copyId?: boolean
	support?: boolean
	manageBilling?: boolean
	resubscribe?: boolean
}

const noticeButtons = computed<NoticeButtons | null>(() => {
	switch (noticeType.value) {
		case 'cancelled':
		case 'setToCancel':
			return { downloadBackup: true, copyId: true, support: true, resubscribe: true }
		case 'paymentfailed':
			return { downloadBackup: true, copyId: true, support: true, manageBilling: true }
		case 'moderated':
		case 'suspended':
			return { copyId: true, support: true }
		default:
			return null
	}
})

const hasNotice = computed(() => !!noticeType.value || !!props.pendingChange)

const showGameLabel = computed(() => !!props.game && !isConfiguring.value)
const showLoaderLabel = computed(() => !!props.loader && !isConfiguring.value)
const showPlayerCount = computed(() => !!props.playerCount && !isConfiguring.value)

const { data: projectData } = useQuery({
	queryKey: ['project', props.upstream?.project_id] as const,
	queryFn: async () => {
		if (!props.upstream?.project_id) return null
		return await labrinth.projects_v2.get(props.upstream.project_id)
	},
	enabled: computed(() => !!props.upstream?.project_id),
})

const iconUrl = computed(() => projectData.value?.icon_url)

async function processImageBlob(blob: Blob, size: number): Promise<string> {
	return new Promise((resolve) => {
		const canvas = document.createElement('canvas')
		const ctx = canvas.getContext('2d')!
		const img = new Image()
		img.onload = () => {
			canvas.width = size
			canvas.height = size
			ctx.drawImage(img, 0, 0, size, size)
			const dataURL = canvas.toDataURL('image/png')
			URL.revokeObjectURL(img.src)
			resolve(dataURL)
		}
		img.src = URL.createObjectURL(blob)
	})
}

async function dataURLToBlob(dataURL: string): Promise<Blob> {
	const res = await fetch(dataURL)
	return res.blob()
}

const { data: image } = useQuery({
	queryKey: ['server-icon', props.server_id] as const,
	queryFn: async (): Promise<string | null> => {
		if (!props.server_id || props.status !== 'available') return null

		try {
			try {
				const blob = await kyros.files_v0.downloadFile('/server-icon-original.png')

				return await processImageBlob(blob, 512)
			} catch {
				const projectIcon = iconUrl.value
				if (projectIcon) {
					const response = await fetch(projectIcon)
					const blob = await response.blob()

					const scaledDataUrl = await processImageBlob(blob, 64)
					const scaledBlob = await dataURLToBlob(scaledDataUrl)
					const scaledFile = new File([scaledBlob], 'server-icon.png', { type: 'image/png' })

					kyros.files_v0.uploadFile('/server-icon.png', scaledFile)

					const originalFile = new File([blob], 'server-icon-original.png', {
						type: 'image/png',
					})
					kyros.files_v0.uploadFile('/server-icon-original.png', originalFile)

					return scaledDataUrl
				}
			}
		} catch (error) {
			console.debug('Icon processing failed:', error)
			return null
		}

		return null
	},
	enabled: computed(() => !!props.server_id && props.status === 'available'),
})

const copied = ref(false)

async function copyToClipboard(text: string) {
	await navigator.clipboard.writeText(text)
	copied.value = true
	setTimeout(() => {
		copied.value = false
	}, 3000)
}
</script>

<style scoped>
.server-listing-notice {
	@apply relative flex w-full rounded-b-2xl border-[1px] border-solid p-4 flex-col gap-4 border-surface-5 bg-bg-raised text-primary;
}
</style>
