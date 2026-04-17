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
				<div v-else class="flex flex-wrap items-center gap-2">
					<div v-if="props.server?.loader" class="flex items-center gap-2 font-medium capitalize">
						<LoaderIcon :loader="props.server.loader" class="flex shrink-0 [&&]:size-5" />
						{{ props.server.loader }} {{ props.server.mc_version }}
					</div>

					<div
						v-if="
							props.server?.loader &&
							props.server?.net?.domain &&
							!userPreferences.hideSubdomainLabel
						"
						class="h-1.5 w-1.5 rounded-full bg-surface-5"
					/>

					<div
						v-if="props.server?.net?.domain && !userPreferences.hideSubdomainLabel"
						v-tooltip="'Copy server address'"
						class="flex cursor-pointer items-center gap-2 font-medium hover:underline text-nowrap"
						@click="copyServerAddress"
					>
						<LinkIcon class="flex size-5 shrink-0" />
						{{ props.server.net.domain }}.modrinth.gg
					</div>

					<div v-if="showUptime" class="h-1.5 w-1.5 rounded-full bg-surface-5" />

					<div v-if="showUptime" class="flex items-center gap-2 font-medium">
						<TimerIcon class="flex size-5 shrink-0" />
						{{ formattedUptime }}
					</div>

					<div
						v-if="showProject && (props.server?.loader || props.server?.net?.domain || showUptime)"
						class="h-1.5 w-1.5 rounded-full bg-surface-5"
					/>

					<div
						v-if="showProject"
						class="flex items-center gap-1.5 font-medium text-primary text-nowrap"
					>
						Linked to
						<Avatar
							:src="props.serverProject?.icon_url ?? undefined"
							:alt="props.serverProject?.title ?? ''"
							size="24px"
						/>
						<AutoLink :to="serverProjectLink" class="truncate text-primary hover:underline">
							{{ props.serverProject?.title }}
						</AutoLink>
					</div>
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
import { LinkIcon, LoaderIcon, SettingsIcon, TimerIcon } from '@modrinth/assets'
import { useStorage } from '@vueuse/core'
import { computed } from 'vue'

import { AutoLink, Avatar, ContentPageHeader, ServerIcon } from '#ui/components'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

type ServerProjectSummary = {
	id: string
	slug?: string | null
	title: string
	icon_url?: string | null
}

const props = withDefaults(
	defineProps<{
		server: Archon.Servers.v0.Server | null | undefined
		serverImage?: string | null
		serverProject?: ServerProjectSummary | null
		serverProjectLink?: string
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
		uptimeSeconds: 0,
		showUptime: true,
		backHref: '/hosting/manage',
		breadcrumbClass: 'breadcrumb goto-link flex w-fit items-center',
		headerClass: '',
	},
)

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { serverId } = injectModrinthServerContext()
const isNuxt = computed(() => client instanceof NuxtModrinthClient)

const userPreferences = useStorage(`pyro-server-${serverId}-preferences`, {
	hideSubdomainLabel: false,
})

const headerImage = computed(() => {
	if (props.server?.is_medal) {
		return 'https://cdn-raw.modrinth.com/medal_icon.webp'
	}
	return props.serverImage ?? undefined
})

const showUptime = computed(() => props.showUptime && (props.uptimeSeconds ?? 0) > 0)

const formattedUptime = computed(() => {
	const uptime = props.uptimeSeconds ?? 0
	const days = Math.floor(uptime / (24 * 3600))
	const hours = Math.floor((uptime % (24 * 3600)) / 3600)
	const minutes = Math.floor((uptime % 3600) / 60)
	const seconds = uptime % 60

	let formatted = ''
	if (days > 0) formatted += `${days}d `
	if (hours > 0 || days > 0) formatted += `${hours}h `
	formatted += `${minutes}m ${seconds}s`
	return formatted.trim()
})

const showProject = computed(() => !!props.serverProject)

const serverProjectLink = computed(() => {
	if (props.serverProjectLink) {
		return props.serverProjectLink
	}
	if (!props.serverProject) {
		return ''
	}
	return `/project/${props.serverProject.slug ?? props.serverProject.id}`
})

function copyServerAddress() {
	if (!props.server?.net?.domain) return
	navigator.clipboard.writeText(`${props.server.net.domain}.modrinth.gg`)
	addNotification({
		title: 'Server address copied',
		text: "Your server's address has been copied to your clipboard.",
		type: 'success',
	})
}
</script>
