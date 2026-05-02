<template>
	<div
		class="transition-all"
		:class="{
			pressable: !isDisabled,
			hoverable: !isDisabled,
			'cursor-pointer': !isDisabled,
		}"
		:role="!isDisabled ? 'link' : undefined"
		:tabindex="!isDisabled ? 0 : undefined"
		@click="navigateToServer"
		@keydown.enter.self="navigateToServer"
		@keydown.space.prevent.self="navigateToServer"
	>
		<div
			class="flex flex-row items-center overflow-x-hidden rounded-2xl border-[1px] border-solid border-surface-4 bg-bg-raised p-4 transition-all duration-150"
			:class="{
				'!rounded-b-none border-b-0': hasNotice,
				'bg-surface-2': isDisabled,
			}"
			data-pyro-server-listing
			:data-pyro-server-listing-id="server_id"
		>
			<div
				v-if="hasIconOverlay"
				class="flex size-16 items-center justify-center rounded-xl border-[1px] border-solid border-button-border bg-button-bg shadow-sm"
			>
				<ServerIcon :image="image ?? undefined" :disabled="isDisabled" class="!rounded-xl" />
				<SpinnerIcon
					v-if="isProvisioning || isUpgrading"
					class="size-8 animate-spin absolute text-contrast"
					:class="{ 'opacity-50': isDisabled }"
				/>
				<LockIcon v-else class="size-8 absolute" :class="{ 'opacity-50': isDisabled }" />
			</div>
			<ServerIcon v-else :image="image ?? undefined" :disabled="isDisabled" />
			<div class="ml-4 flex flex-col gap-1.5">
				<div class="flex flex-row items-center gap-2">
					<h2 class="m-0 text-xl font-bold text-contrast" :class="{ 'opacity-50': isDisabled }">
						{{ name }}
					</h2>
					<div
						v-if="isConfiguring && noticeType !== 'cancelled' && noticeType !== 'setToCancel'"
						class="flex min-w-0 items-center gap-2 truncate text-sm font-medium text-brand rounded-full bg-brand-highlight border border-solid border-brand px-2.5 h-[28px]"
					>
						<SparklesIcon class="size-5 shrink-0 font-semibold" />
						{{ formatMessage(messages.newLabel) }}
					</div>
				</div>

				<div
					v-if="projectData?.title"
					class="m-0 flex flex-row items-center gap-2 text-sm font-medium"
					:class="{ 'opacity-50': isDisabled }"
				>
					<Avatar
						:src="iconUrl"
						no-shadow
						style="min-height: 20px; min-width: 20px; height: 20px; width: 20px"
						:alt="formatMessage(messages.serverIconAlt)"
					/>
					{{ formatMessage(messages.usingProjectLabel, { projectTitle: projectData?.title }) }}
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
					:server-id="server_id"
					:show-game-label="showGameLabel"
					:show-loader-label="showLoaderLabel"
					:show-player-count="showPlayerCount"
					:class="{ 'opacity-50': isDisabled }"
					:linked="false"
					class="flex w-full flex-row flex-wrap items-center gap-2 text-primary *:hidden sm:flex-row sm:*:flex"
				/>
			</div>
		</div>

		<div v-if="noticeType" class="server-listing-notice">
			<div v-if="noticeType === 'provisioning'" class="flex gap-2">
				{{ formatMessage(messages.provisioningNotice) }}
			</div>
			<div v-else-if="noticeType === 'upgrading'" class="flex gap-2">
				{{ formatMessage(messages.upgradingNotice) }}
			</div>
			<div v-else-if="noticeType === 'cancelled' || noticeType === 'paymentfailed'">
				<IntlFormatted
					v-if="noticeType === 'paymentfailed' && cancellationDate"
					:message-id="messages.subscriptionCancelledPaymentFailedOnDate"
					:values="{ formattedDate: formatDate(cancellationDate) }"
				>
					<template #date="{ children }">
						<span class="font-medium text-contrast"><component :is="() => children" /></span>
					</template>
				</IntlFormatted>

				<span v-else-if="noticeType === 'paymentfailed'">
					{{ formatMessage(messages.subscriptionCancelledPaymentFailed) }}
				</span>

				<IntlFormatted
					v-else-if="cancellationDate"
					:message-id="messages.subscriptionCancelledOnDate"
					:values="{ formattedDate: formatDate(cancellationDate) }"
				>
					<template #date="{ children }">
						<span class="font-medium text-contrast"><component :is="() => children" /></span>
					</template>
				</IntlFormatted>

				<span v-else>
					{{ formatMessage(messages.subscriptionCancelled) }}
				</span>

				{{ ' ' }}
				<IntlFormatted
					v-if="!isFilesExpired"
					:message-id="messages.filesKeptForDownload"
					:values="{ daysRemaining: filesRemainingDays }"
				>
					<template #days-remaining="{ children }">
						<span class="font-medium text-red">
							<component :is="() => children" />
						</span>
					</template>
				</IntlFormatted>
			</div>

			<div v-else-if="noticeType === 'setToCancel'">
				<IntlFormatted
					v-if="cancellationDate"
					:message-id="messages.subscriptionSetToCancelOnDate"
					:values="{ formattedDate: formatDate(cancellationDate) }"
				>
					<template #date="{ children }">
						<span class="font-medium text-contrast">
							<component :is="() => children" />
						</span>
					</template>
				</IntlFormatted>

				<span v-else>{{ formatMessage(messages.subscriptionSetToCancel) }}</span>

				<template v-if="!isFilesExpired">
					{{ ' ' }}
					{{ formatMessage(messages.filesPreservedAfterCancellation) }}
				</template>
			</div>
			<div v-else-if="noticeType === 'moderated'">
				{{ formatMessage(messages.moderatedNotice) }}
			</div>
			<div v-else>
				{{ formatMessage(messages.suspendedNotice) }}
			</div>

			<div v-if="noticeButtons" class="flex gap-2">
				<ButtonStyled
					v-if="noticeButtons.downloadBackup && onDownloadBackup && isBackupDownloadEnabled"
					type="outlined"
					circular
				>
					<button
						v-tooltip="formatMessage(messages.downloadLatestBackupTooltip)"
						class="!border-surface-4"
						data-server-listing-button
						@click="onDownloadBackup"
					>
						<DownloadIcon />
					</button>
				</ButtonStyled>
				<ButtonStyled v-if="noticeButtons.copyId" type="outlined">
					<button
						v-tooltip="formatMessage(messages.copyCodeToClipboardTooltip)"
						class="!border-surface-4"
						data-server-listing-button
						@click="copyToClipboard(server_id)"
					>
						<template v-if="copied">
							{{ formatMessage(messages.copiedLabel) }} <CheckIcon class="text-green" />
						</template>
						<template v-else> {{ formatMessage(messages.copyIdLabel) }} <CopyIcon /> </template>
					</button>
				</ButtonStyled>
				<ButtonStyled v-if="noticeButtons.support">
					<a href="https://support.modrinth.com/en/" target="_blank" data-server-listing-button
						><MessagesSquareIcon /> {{ formatMessage(messages.supportLabel) }}
					</a>
				</ButtonStyled>
				<ButtonStyled v-if="noticeButtons.manageBilling" color="brand">
					<AutoLink :to="`/settings/billing#server-${server_id}`" data-server-listing-button>
						<CardIcon /> {{ formatMessage(messages.manageBillingLabel) }}
					</AutoLink>
				</ButtonStyled>
				<ButtonStyled v-if="noticeButtons.resubscribe && onResubscribe" color="brand">
					<button data-server-listing-button @click="onResubscribe">
						<RotateCounterClockwiseIcon /> {{ formatMessage(messages.resubscribeLabel) }}
					</button>
				</ButtonStyled>
			</div>
		</div>

		<div v-if="pendingChange && status !== 'suspended'" class="server-listing-notice">
			<div>
				<IntlFormatted
					:message-id="messages.pendingChangeNotice"
					:values="{
						verb: pendingChange.verb.toLowerCase(),
						planSize: pendingChange.planSize,
						formattedDate: formatDate(pendingChange.date),
					}"
				>
					<template #date="{ children }">
						<span class="font-medium text-contrast"><component :is="() => children" /></span>
					</template>
				</IntlFormatted>
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
import { useRouter } from 'vue-router'

import {
	CardIcon,
	CheckIcon,
	CopyIcon,
	RotateCounterClockwiseIcon,
} from '../../../../assets/generated-icons'
import { useFormatDateTime } from '../../composables'
import { defineMessages, useVIntl } from '../../composables/i18n'
import { injectModrinthClient } from '../../providers/api-client'
import Avatar from '../base/Avatar.vue'
import IntlFormatted from '../base/IntlFormatted.vue'
import ServersSpecs from '../billing/ServersSpecs.vue'
import ServerIcon from './icons/ServerIcon.vue'
import ServerInfoLabels from './labels/ServerInfoLabels.vue'

const formatDate = useFormatDateTime({ dateStyle: 'long' })
const { formatMessage } = useVIntl()

const messages = defineMessages({
	newLabel: {
		id: 'servers.listing.new-label',
		defaultMessage: 'New',
	},
	serverIconAlt: {
		id: 'servers.listing.server-icon-alt',
		defaultMessage: 'Server icon',
	},
	usingProjectLabel: {
		id: 'servers.listing.using-project-label',
		defaultMessage: 'Using {projectTitle}',
	},
	provisioningNotice: {
		id: 'servers.listing.notice.provisioning',
		defaultMessage: 'Please wait while we set up your server. This can take up to 10 minutes.',
	},
	upgradingNotice: {
		id: 'servers.listing.notice.upgrading',
		defaultMessage:
			"Your server's hardware is currently being upgraded and will be back online shortly.",
	},
	subscriptionCancelled: {
		id: 'servers.listing.notice.subscription-cancelled',
		defaultMessage: 'Your subscription was cancelled.',
	},
	subscriptionCancelledOnDate: {
		id: 'servers.listing.notice.subscription-cancelled-on-date',
		defaultMessage: 'Your subscription was cancelled on <date>{formattedDate}</date>. ',
	},
	subscriptionCancelledPaymentFailed: {
		id: 'servers.listing.notice.subscription-cancelled-payment-failed',
		defaultMessage: 'Your subscription was cancelled due to payment failure.',
	},
	subscriptionCancelledPaymentFailedOnDate: {
		id: 'servers.listing.notice.subscription-cancelled-payment-failed-on-date',
		defaultMessage:
			'Your subscription was cancelled on <date>{formattedDate}</date> due to payment failure. ',
	},
	filesKeptForDownload: {
		id: 'servers.listing.notice.files-kept-for-download',
		defaultMessage:
			'Your files will be kept for <days-remaining>{daysRemaining} more {daysRemaining, plural, one {day} other {days} }</days-remaining>. Contact support to download the files before they are deleted. ',
	},
	subscriptionSetToCancel: {
		id: 'servers.listing.notice.subscription-set-to-cancel',
		defaultMessage: 'Your subscription is set to cancel.',
	},
	subscriptionSetToCancelOnDate: {
		id: 'servers.listing.notice.subscription-set-to-cancel-on-date',
		defaultMessage: 'Your subscription is set to cancel on <date>{formattedDate}</date>. ',
	},
	filesPreservedAfterCancellation: {
		id: 'servers.listing.notice.files-preserved-after-cancellation',
		defaultMessage: 'Your files will be preserved for 30 days after cancellation.',
	},
	moderatedNotice: {
		id: 'servers.listing.notice.moderated',
		defaultMessage: 'Your server has been suspended by moderation action. ',
	},
	suspendedNotice: {
		id: 'servers.listing.notice.suspended',
		defaultMessage:
			'Your server has been suspended. Please contact Modrinth Support for more information.',
	},
	downloadLatestBackupTooltip: {
		id: 'servers.listing.download-latest-backup-tooltip',
		defaultMessage: 'Download latest backup',
	},
	copyCodeToClipboardTooltip: {
		id: 'servers.listing.copy-code-tooltip',
		defaultMessage: 'Copy code to clipboard',
	},
	copiedLabel: {
		id: 'servers.listing.copied-label',
		defaultMessage: 'Copied',
	},
	copyIdLabel: {
		id: 'servers.listing.copy-id-label',
		defaultMessage: 'Copy ID',
	},
	supportLabel: {
		id: 'servers.listing.support-label',
		defaultMessage: 'Support',
	},
	manageBillingLabel: {
		id: 'servers.listing.manage-billing-label',
		defaultMessage: 'Manage billing',
	},
	resubscribeLabel: {
		id: 'servers.listing.resubscribe-label',
		defaultMessage: 'Resubscribe',
	},
	pendingChangeNotice: {
		id: 'servers.listing.notice.pending-change',
		defaultMessage:
			'Your server will {verb} to the {planSize} Plan on <date>{formattedDate}</date>. ',
	},
})

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
	isProvisioning?: boolean
	cancellationDate?: string | Date | null
	onResubscribe?: (() => void) | null
	onDownloadBackup?: (() => void) | null
}

const props = defineProps<ServerListingProps>()
const router = useRouter()

const { archon, kyros, labrinth } = injectModrinthClient()

const isBackupDownloadEnabled = false
const isConfiguring = computed(() => props.flows?.intro)
const isUpgrading = computed(
	() => props.status === 'suspended' && props.suspension_reason === 'upgrading',
)
const isDisabled = computed(() => props.status === 'suspended' || props.isProvisioning)
const isSetToCancel = computed(() => !!props.cancellationDate && props.status !== 'suspended')
const filesRemainingDays = computed(() => {
	if (!props.cancellationDate) return 0
	const cancellation = new Date(props.cancellationDate)
	const expiresAt = new Date(cancellation.getTime() + 30 * 24 * 60 * 60 * 1000) // expires 30 days after cancellation
	const remaining = Math.ceil((expiresAt.getTime() - Date.now()) / (24 * 60 * 60 * 1000))
	return Math.max(0, remaining)
})
const isFilesExpired = computed(() => filesRemainingDays.value <= 0)

const hasIconOverlay = computed(
	() => props.isProvisioning || isUpgrading.value || props.status === 'suspended',
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
	if (props.isProvisioning) return 'provisioning'
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
			return { downloadBackup: true, copyId: true, support: true }
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
			const fsAuth = await archon.servers_v0.getFilesystemAuth(props.server_id)

			try {
				const blob = await kyros.files_v0.downloadFileWithAuth(fsAuth, '/server-icon.png')
				return await processImageBlob(blob, 64)
			} catch (error) {
				const statusCode = (error as { statusCode?: number })?.statusCode
				if (statusCode != null && statusCode !== 404) {
					throw error
				}

				try {
					const originalBlob = await kyros.files_v0.downloadFileWithAuth(
						fsAuth,
						'/server-icon-original.png',
					)
					return await processImageBlob(originalBlob, 64)
				} catch (originalError) {
					const originalStatusCode = (originalError as { statusCode?: number })?.statusCode
					if (originalStatusCode != null && originalStatusCode !== 404) {
						throw originalError
					}
				}

				const projectIcon = iconUrl.value
				if (projectIcon) {
					const response = await fetch(projectIcon)
					const blob = await response.blob()

					const scaledDataUrl = await processImageBlob(blob, 64)
					const scaledBlob = await dataURLToBlob(scaledDataUrl)
					const scaledFile = new File([scaledBlob], 'server-icon.png', { type: 'image/png' })

					await kyros.files_v0.uploadFileWithAuth(fsAuth, '/server-icon.png', scaledFile).promise

					const originalFile = new File([blob], 'server-icon-original.png', {
						type: 'image/png',
					})
					await kyros.files_v0.uploadFileWithAuth(fsAuth, '/server-icon-original.png', originalFile)
						.promise

					return scaledDataUrl
				}
			}

			return null
		} catch (error) {
			console.debug('Icon processing failed:', error)
			return null
		}
	},
	enabled: computed(() => !!props.server_id && props.status === 'available'),
})

const copied = ref(false)

function navigateToServer(event: MouseEvent | KeyboardEvent) {
	if (isDisabled.value) return

	const target = event.target
	if (
		target instanceof HTMLElement &&
		target.closest('[data-subdomain-label], [data-server-listing-button]')
	) {
		return
	}

	router.push(`/hosting/manage/${props.server_id}`)
}

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
	@apply relative flex w-full rounded-b-2xl border-[1px] border-solid p-4 flex-col gap-4 border-surface-4 bg-bg-raised text-primary;
}

.hoverable:hover:not(:has([data-subdomain-label]:hover, [data-server-listing-button]:hover)) {
	filter: brightness(1.2);
}

.pressable:active:not(:has([data-subdomain-label]:active, [data-server-listing-button]:active)) {
	transform: scale(0.985);
}
</style>
