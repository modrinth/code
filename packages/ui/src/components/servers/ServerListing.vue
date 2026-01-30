<template>
	<div>
		<NuxtLink :to="status === 'suspended' ? '' : `/hosting/manage/${props.server_id}`">
			<div
				class="flex flex-row items-center overflow-x-hidden rounded-2xl border-[1px] border-solid border-button-bg bg-bg-raised p-4 transition-transform duration-100"
				:class="{
					'!rounded-b-none border-b-0': status === 'suspended' || !!pendingChange,
					'opacity-75': status === 'suspended',
					'active:scale-95': status !== 'suspended' && !pendingChange,
				}"
				data-pyro-server-listing
				:data-pyro-server-listing-id="server_id"
			>
				<ServerIcon v-if="status !== 'suspended'" :image="image" />
				<div
					v-else
					class="bg-bg-secondary flex size-16 items-center justify-center rounded-xl border-[1px] border-solid border-button-border bg-button-bg shadow-sm"
				>
					<LockIcon class="size-12 text-secondary" />
				</div>
				<div class="ml-4 flex flex-col gap-2.5">
					<div class="flex flex-row items-center gap-2">
						<h2 class="m-0 text-xl font-bold text-contrast">{{ name }}</h2>
						<ChevronRightIcon />
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

					<div
						v-if="isConfiguring"
						class="flex min-w-0 items-center gap-2 truncate text-sm font-semibold text-brand"
					>
						<SparklesIcon class="size-5 shrink-0" /> New server
					</div>
					<ServerInfoLabels
						v-else
						:server-data="{ game, mc_version, loader, loader_version, net }"
						:show-game-label="showGameLabel"
						:show-loader-label="showLoaderLabel"
						:linked="false"
						class="pointer-events-none flex w-full flex-row flex-wrap items-center gap-4 text-secondary *:hidden sm:flex-row sm:*:flex"
					/>
				</div>
			</div>
		</NuxtLink>
		<div
			v-if="status === 'suspended' && suspension_reason === 'upgrading'"
			class="relative flex w-full flex-row items-center gap-2 rounded-b-2xl border-[1px] border-t-0 border-solid border-bg-blue bg-bg-blue p-4 text-sm font-bold text-contrast"
		>
			<LoaderCircleIcon class="size-5 animate-spin" />
			Your server's hardware is currently being upgraded and will be back online shortly.
		</div>
		<div
			v-else-if="status === 'suspended' && suspension_reason === 'cancelled'"
			class="relative flex w-full flex-col gap-2 rounded-b-2xl border-[1px] border-t-0 border-solid border-bg-red bg-bg-red p-4 text-sm font-bold text-contrast"
		>
			<div class="flex flex-row gap-2">
				<TriangleAlertIcon class="!size-5" /> Your server has been cancelled. Please update your
				billing information or contact Modrinth Support for more information.
			</div>
			<CopyCode :text="`${props.server_id}`" class="ml-auto" />
		</div>
		<div
			v-else-if="status === 'suspended' && suspension_reason"
			class="relative flex w-full flex-col gap-2 rounded-b-2xl border-[1px] border-t-0 border-solid border-bg-red bg-bg-red p-4 text-sm font-bold text-contrast"
		>
			<div class="flex flex-row gap-2">
				<TriangleAlertIcon class="!size-5" /> Your server has been suspended:
				{{ suspension_reason }}. Please update your billing information or contact Modrinth Support
				for more information.
			</div>
			<CopyCode :text="`${props.server_id}`" class="ml-auto" />
		</div>
		<div
			v-else-if="status === 'suspended'"
			class="relative flex w-full flex-col gap-2 rounded-b-2xl border-[1px] border-t-0 border-solid border-bg-red bg-bg-red p-4 text-sm font-bold text-contrast"
		>
			<div class="flex flex-row gap-2">
				<TriangleAlertIcon class="!size-5" /> Your server has been suspended. Please update your
				billing information or contact Modrinth Support for more information.
			</div>
			<CopyCode :text="`${props.server_id}`" class="ml-auto" />
		</div>
		<div
			v-if="pendingChange && status !== 'suspended'"
			class="relative flex w-full flex-col gap-2 rounded-b-2xl border-[1px] border-t-0 border-solid border-orange bg-bg-orange p-4 text-sm font-bold text-contrast"
		>
			<div>
				Your server will {{ pendingChange.verb.toLowerCase() }} to the "{{
					pendingChange.planSize
				}}" plan on {{ formatDate(pendingChange.date) }}.
			</div>
			<ServersSpecs
				class="!font-normal !text-contrast"
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
	ChevronRightIcon,
	LoaderCircleIcon,
	LockIcon,
	SparklesIcon,
	TriangleAlertIcon,
} from '@modrinth/assets'
import { useQuery } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import { computed } from 'vue'

import { injectModrinthClient } from '../../providers/api-client'
import Avatar from '../base/Avatar.vue'
import CopyCode from '../base/CopyCode.vue'
import ServersSpecs from '../billing/ServersSpecs.vue'
import ServerIcon from './icons/ServerIcon.vue'
import ServerInfoLabels from './labels/ServerInfoLabels.vue'

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
}

const props = defineProps<ServerListingProps>()

const { archon, kyros, labrinth } = injectModrinthClient()

const showGameLabel = computed(() => !!props.game)
const showLoaderLabel = computed(() => !!props.loader)

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
	queryFn: async (): Promise<string | undefined> => {
		if (!props.server_id || props.status !== 'available') return undefined

		try {
			const auth = await archon.servers_v0.getFilesystemAuth(props.server_id)

			try {
				const blob = await kyros.files_v0.downloadFile(
					auth.url,
					auth.token,
					'/server-icon-original.png',
				)

				return await processImageBlob(blob, 512)
			} catch {
				const projectIcon = iconUrl.value
				if (projectIcon) {
					const response = await fetch(projectIcon)
					const blob = await response.blob()

					const scaledDataUrl = await processImageBlob(blob, 64)
					const scaledBlob = await dataURLToBlob(scaledDataUrl)
					const scaledFile = new File([scaledBlob], 'server-icon.png', { type: 'image/png' })

					await kyros.files_v0.uploadFile(auth.url, auth.token, '/server-icon.png', scaledFile)

					const originalFile = new File([blob], 'server-icon-original.png', {
						type: 'image/png',
					})
					await kyros.files_v0.uploadFile(
						auth.url,
						auth.token,
						'/server-icon-original.png',
						originalFile,
					)

					return scaledDataUrl
				}
			}
		} catch (error) {
			console.debug('Icon processing failed:', error)
			return undefined
		}
	},
	enabled: computed(() => !!props.server_id && props.status === 'available'),
})

const isConfiguring = computed(() => props.flows?.intro)

const formatDate = (d: unknown) => {
	try {
		return dayjs(d as string).format('MMMM D, YYYY')
	} catch {
		return ''
	}
}
</script>
