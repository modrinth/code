<template>
	<div class="w-full flex flex-col gap-4" :class="{ 'mt-4': isNuxt }">
		<ContentPageHeader :class="props.headerClass">
			<template #icon>
				<ServerIcon
					:image="headerImage"
					:class="isNuxt ? 'size-20 !rounded-2xl' : 'size-16 !rounded-xl'"
				/>
			</template>
			<template #title>
				{{ props.server?.name || 'Server' }}
			</template>
			<template #stats>
				<div
					v-if="props.server?.flows?.intro"
					class="flex items-center gap-2 font-semibold text-secondary"
				>
					<SettingsIcon />
					Configuring server...
				</div>
				<div v-else class="flex min-w-0 flex-wrap items-center gap-2">
					<template v-for="(item, index) in headerStats" :key="item.id">
						<div v-if="index > 0" class="h-1.5 w-1.5 rounded-full bg-surface-5" />
						<button
							v-if="item.copyable"
							v-tooltip="'Copy server address'"
							class="m-0 flex min-w-0 cursor-pointer items-center gap-2 border-0 bg-transparent p-0 font-medium text-secondary hover:underline text-nowrap"
							type="button"
							@click="copyServerAddress"
						>
							<component :is="item.icon" class="flex size-5 shrink-0" />
							<span class="truncate">{{ item.label }}</span>
						</button>
						<div
							v-else
							class="flex min-w-0 items-center gap-2 font-medium text-secondary text-nowrap"
						>
							<component :is="item.icon" class="flex size-5 shrink-0" />
							<span class="truncate">{{ item.label }}</span>
						</div>
					</template>
				</div>
			</template>
			<template #actions>
				<slot name="actions" />
			</template>
		</ContentPageHeader>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { NuxtModrinthClient } from '@modrinth/api-client'
import { GlobeIcon, LinkIcon, SettingsIcon } from '@modrinth/assets'
import type { Component } from 'vue'
import { computed } from 'vue'

import { ContentPageHeader, ServerIcon } from '#ui/components'
import { injectModrinthClient, injectNotificationManager } from '#ui/providers'

type ServerProjectSummary = {
	id: string
	slug?: string | null
	title: string
	icon_url?: string | null
}

type HeaderStat = {
	id: string
	label: string
	icon: Component
	copyable?: boolean
}

const props = withDefaults(
	defineProps<{
		server: Archon.Servers.v0.Server | null | undefined
		serverImage?: string | null
		serverProject?: ServerProjectSummary | null
		serverProjectLink?: string
		activeWorldName?: string | null
		uptimeSeconds?: number
		showUptime?: boolean
		backHref?: string
		breadcrumbClass?: string
		headerClass?: string
	}>(),
	{
		serverImage: null,
		serverProject: null,
		serverProjectLink: '',
		activeWorldName: null,
		uptimeSeconds: 0,
		showUptime: true,
		backHref: '/hosting/manage',
		breadcrumbClass: 'breadcrumb goto-link flex w-fit items-center',
		headerClass: '',
	},
)

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const isNuxt = computed(() => client instanceof NuxtModrinthClient)

const headerImage = computed(() => {
	if (props.server?.is_medal) {
		return 'https://cdn-raw.modrinth.com/medal_icon.webp'
	}
	return props.serverImage ?? undefined
})

const serverAddress = computed(() => {
	const domain = props.server?.net?.domain
	if (domain) return `${domain}.modrinth.gg`

	const ip = props.server?.net?.ip
	if (!ip) return null
	const port = props.server?.net?.port
	return port ? `${ip}:${port}` : ip
})

const headerStats = computed<HeaderStat[]>(() => {
	const stats: HeaderStat[] = []
	const worldName = props.activeWorldName
	if (worldName) {
		stats.push({
			id: 'world',
			label: worldName,
			icon: GlobeIcon,
		})
	}
	if (serverAddress.value) {
		stats.push({
			id: 'address',
			label: serverAddress.value,
			icon: LinkIcon,
			copyable: true,
		})
	}
	return stats
})

function copyServerAddress() {
	if (!serverAddress.value) return
	navigator.clipboard.writeText(serverAddress.value)
	addNotification({
		title: 'Server address copied',
		text: "Your server's address has been copied to your clipboard.",
		type: 'success',
	})
}
</script>
