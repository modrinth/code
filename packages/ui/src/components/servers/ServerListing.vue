<template>
	<div>
		<NuxtLink :to="status === 'suspended' ? '' : `/hosting/manage/${server_id}`">
			<div
				class="flex flex-row items-center overflow-x-hidden rounded-2xl border-[1px] border-solid border-surface-5 bg-bg-raised p-4 transition-transform duration-100"
				:class="{
					'!rounded-b-none border-b-0': status === 'suspended' || !!pendingChange,
					'opacity-50 bg-surface-2': status === 'suspended',
					'active:scale-95': status !== 'suspended' && !pendingChange,
				}"
				data-pyro-server-listing
				:data-pyro-server-listing-id="server_id"
			>
				<ServerIcon v-if="status !== 'suspended'" :image="image ?? undefined" />
				<div
					v-else
					class="bg-bg-secondary flex size-16 items-center justify-center rounded-xl border-[1px] border-solid border-button-border bg-button-bg shadow-sm"
				>
					<LockIcon class="size-12 text-secondary" />
				</div>
				<div class="ml-4 flex flex-col gap-2.5">
					<div class="flex flex-row items-center gap-2">
						<h2 class="m-0 text-xl font-bold text-contrast">{{ name }}</h2>
						<div
							v-if="isConfiguring"
							class="flex min-w-0 items-center gap-2 truncate text-sm font-medium text-brand rounded-full bg-brand-highlight border border-solid border-brand px-2.5 h-[28px]"
						>
							<SparklesIcon class="size-5 shrink-0 font-semibold" /> New
						</div>
					</div>

					<div
						v-if="projectData?.title"
						class="m-0 flex flex-row items-center gap-2 text-sm font-medium text-secondary"
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
						:linked="false"
						class="pointer-events-none flex w-full flex-row flex-wrap items-center gap-2 text-primary *:hidden sm:flex-row sm:*:flex"
					/>
				</div>
			</div>
		</NuxtLink>
		<div
			v-if="status === 'suspended' && suspension_reason === 'upgrading'"
			class="server-listing-notice"
		>
			<div class="flex gap-2">
				<LoaderCircleIcon class="size-5 animate-spin" />
				Your server's hardware is currently being upgraded and will be back online shortly.
			</div>
		</div>
		<div
			v-else-if="status === 'suspended' && suspension_reason === 'cancelled'"
			class="server-listing-notice"
		>
			<div class="flex flex-row gap-2">
				Your server has been cancelled. Please update your billing information or contact Modrinth
				Support for more information.
			</div>
			<div class="flex gap-2">
				<ButtonStyled type="outlined" @click="copyToClipboard(server_id)">
					<button
						class="!border-surface-5"
						v-tooltip="'Copy code to clipboard'"
						@click="copyToClipboard(server_id)"
					>
						<template v-if="copied"> Copied <CheckIcon class="text-green" /> </template>
						<template v-else> Copy ID <CopyIcon /> </template>
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<a href="https://support.modrinth.com/en/" target="_blank"
						><MessagesSquareIcon /> Support
					</a>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<AutoLink :to="`/settings/billing#server-${server_id}`">
						<RotateCounterClockwiseIcon /> Resubscribe
					</AutoLink>
				</ButtonStyled>
			</div>
		</div>
		<div v-else-if="status === 'suspended' && suspension_reason" class="server-listing-notice">
			<div class="flex flex-row gap-2">
				Your server has been suspended:
				{{ suspension_reason }}. Please update your billing information or contact Modrinth Support
				for more information.
			</div>
			<div class="flex gap-2">
				<ButtonStyled type="outlined" @click="copyToClipboard(server_id)">
					<button
						class="!border-surface-5"
						v-tooltip="'Copy code to clipboard'"
						@click="copyToClipboard(server_id)"
					>
						<template v-if="copied"> Copied <CheckIcon class="text-green" /> </template>
						<template v-else> Copy ID <CopyIcon /> </template>
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<a href="https://support.modrinth.com/en/" target="_blank"
						><MessagesSquareIcon /> Support
					</a>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<AutoLink :to="`/settings/billing#server-${server_id}`">
						<CardIcon /> Manage billing
					</AutoLink>
				</ButtonStyled>
			</div>
		</div>
		<div v-else-if="status === 'suspended'" class="server-listing-notice">
			<div class="flex flex-row gap-2">
				Your server has been suspended. Please update your billing information or contact Modrinth
				Support for more information.
			</div>
			<div class="flex gap-2">
				<ButtonStyled type="outlined" @click="copyToClipboard(server_id)">
					<button
						class="!border-surface-5"
						v-tooltip="'Copy code to clipboard'"
						@click="copyToClipboard(server_id)"
					>
						<template v-if="copied"> Copied <CheckIcon class="text-green" /> </template>
						<template v-else> Copy ID <CopyIcon /> </template>
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<a href="https://support.modrinth.com/en/" target="_blank"
						><MessagesSquareIcon /> Support
					</a>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<AutoLink :to="`/settings/billing#server-${server_id}`">
						<CardIcon /> Manage billing
					</AutoLink>
				</ButtonStyled>
			</div>
		</div>
		<div v-if="pendingChange && status !== 'suspended'" class="server-listing-notice">
			<div>
				Your server will {{ pendingChange.verb.toLowerCase() }} to the "{{
					pendingChange.planSize
				}}" plan on {{ formatDate(pendingChange.date) }}.
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
import { LoaderCircleIcon, LockIcon, MessagesSquareIcon, SparklesIcon } from '@modrinth/assets'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { AutoLink, ButtonStyled } from '@modrinth/ui'
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
}

const props = defineProps<ServerListingProps>()

const { kyros, labrinth } = injectModrinthClient()

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

const isConfiguring = computed(() => props.flows?.intro)

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
	@apply relative flex w-full rounded-b-2xl border-[1px] border-solid p-4 flex-col gap-2 border-surface-5 bg-bg-raised text-primary;
}
</style>
