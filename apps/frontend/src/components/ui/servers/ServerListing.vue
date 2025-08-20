<template>
	<div>
		<NuxtLink :to="status === 'suspended' ? '' : `/servers/manage/${props.server_id}`">
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
			<PanelSpinner />
			Your server's hardware is currently being upgraded and will be back online shortly.
		</div>
		<div
			v-else-if="status === 'suspended' && suspension_reason === 'cancelled'"
			class="relative flex w-full flex-col gap-2 rounded-b-2xl border-[1px] border-t-0 border-solid border-bg-red bg-bg-red p-4 text-sm font-bold text-contrast"
		>
			<div class="flex flex-row gap-2">
				<PanelErrorIcon class="!size-5" /> Your server has been cancelled. Please update your
				billing information or contact Modrinth Support for more information.
			</div>
			<CopyCode :text="`${props.server_id}`" class="ml-auto" />
		</div>
		<div
			v-else-if="status === 'suspended' && suspension_reason"
			class="relative flex w-full flex-col gap-2 rounded-b-2xl border-[1px] border-t-0 border-solid border-bg-red bg-bg-red p-4 text-sm font-bold text-contrast"
		>
			<div class="flex flex-row gap-2">
				<PanelErrorIcon class="!size-5" /> Your server has been suspended: {{ suspension_reason }}.
				Please update your billing information or contact Modrinth Support for more information.
			</div>
			<CopyCode :text="`${props.server_id}`" class="ml-auto" />
		</div>
		<div
			v-else-if="status === 'suspended'"
			class="relative flex w-full flex-col gap-2 rounded-b-2xl border-[1px] border-t-0 border-solid border-bg-red bg-bg-red p-4 text-sm font-bold text-contrast"
		>
			<div class="flex flex-row gap-2">
				<PanelErrorIcon class="!size-5" /> Your server has been suspended. Please update your
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
import { ChevronRightIcon, LockIcon, SparklesIcon } from '@modrinth/assets'
import { Avatar, CopyCode, ServersSpecs } from '@modrinth/ui'
import type { Project, Server } from '@modrinth/utils'
import dayjs from 'dayjs'

import { useModrinthServers } from '~/composables/servers/modrinth-servers.ts'

import PanelErrorIcon from './icons/PanelErrorIcon.vue'
import PanelSpinner from './PanelSpinner.vue'
import ServerIcon from './ServerIcon.vue'
import ServerInfoLabels from './ServerInfoLabels.vue'

type PendingChange = {
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

const props = defineProps<Partial<Server> & { pendingChange?: PendingChange }>()

if (props.server_id && props.status === 'available') {
	// Necessary only to get server icon
	await useModrinthServers(props.server_id, ['general'])
}

const showGameLabel = computed(() => !!props.game)
const showLoaderLabel = computed(() => !!props.loader)

let projectData: Ref<Project | null>
if (props.upstream) {
	const { data } = await useAsyncData<Project>(
		`server-project-${props.server_id}`,
		async (): Promise<Project> => {
			const result = await useBaseFetch(`project/${props.upstream?.project_id}`)
			return result as Project
		},
	)
	projectData = data
} else {
	projectData = ref(null)
}

const image = useState<string | undefined>(`server-icon-${props.server_id}`, () => undefined)
const iconUrl = computed(() => projectData.value?.icon_url || undefined)
const isConfiguring = computed(() => props.flows?.intro)

const formatDate = (d: unknown) => {
	try {
		return dayjs(d as any).format('MMMM D, YYYY')
	} catch {
		return ''
	}
}
</script>
