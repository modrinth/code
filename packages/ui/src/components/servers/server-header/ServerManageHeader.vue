<template>
	<div class="w-full flex flex-col gap-4" :class="{ 'mt-4': isNuxt }">
		<PageHeader
			:header="props.server?.name || 'Server'"
			:leading="leadingItems"
			:metadata="metadataItems"
			:actions="headerActions"
			:header-class="props.headerClass"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { NuxtModrinthClient } from '@modrinth/api-client'
import {
	GlobeIcon,
	LinkIcon,
	LoaderCircleIcon,
	PlayIcon,
	SettingsIcon,
	SlashIcon,
	StopCircleIcon,
	TimerIcon,
	UpdatedIcon,
} from '@modrinth/assets'
import { useStorage } from '@vueuse/core'
import type { Component } from 'vue'
import { computed } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import Avatar from '#ui/components/base/Avatar.vue'
import type { JoinedButtonAction } from '#ui/components/base/JoinedButtons.vue'
import PageHeader from '#ui/components/base/PageHeader.vue'
import LoaderIcon from '#ui/components/servers/icons/LoaderIcon.vue'
import ServerIcon from '#ui/components/servers/icons/ServerIcon.vue'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'
import { formatLoaderLabel } from '#ui/utils/loaders'

import { useServerPowerAction } from './use-server-power-action'

type ServerProjectSummary = {
	id: string
	slug?: string | null
	title: string
	icon_url?: string | null
}

type HeaderWorld = {
	id: string
	name: string
}

type HeaderMetadataItem = {
	id: string
	label: string
	icon: Component
	iconProps?: Record<string, unknown>
	tooltip?: string
	ariaLabel?: string
	to?: string | RouteLocationRaw
	onClick?: () => void | Promise<void>
	class?: string
}

type HeaderAction = {
	id: string
	label: string
	icon?: Component
	iconProps?: Record<string, unknown>
	iconClass?: string
	tooltip?: string
	ariaLabel?: string
	to?: string | RouteLocationRaw
	onClick?: () => void | Promise<void>
	disabled?: boolean
	labelHidden?: boolean
	circular?: boolean
	color?: 'standard' | 'brand' | 'red' | 'orange' | 'green' | 'blue' | 'purple'
	size?: 'standard' | 'large' | 'small'
	type?: 'standard' | 'outlined' | 'transparent' | 'highlight' | 'highlight-colored-text' | 'chip'
	joinedActions?: JoinedButtonAction[]
	primaryDisabled?: boolean
	dropdownDisabled?: boolean
	primaryMuted?: boolean
	prompt?: {
		title: string
		description: string
		dismissLabel?: string
		shown?: boolean
		placement?: string
		onDismiss?: () => void
	}
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
		worlds?: HeaderWorld[]
		powerDisabled?: boolean
		settingsLabel?: string
		showSettingsHint?: boolean
		settingsHintTitle?: string
		settingsHintDescription?: string
		settingsHintDismissLabel?: string
		actions?: HeaderAction[]
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
		worlds: () => [],
		powerDisabled: false,
		settingsLabel: 'Server settings',
		showSettingsHint: false,
		settingsHintTitle: '',
		settingsHintDescription: '',
		settingsHintDismissLabel: "Don't show again",
		actions: () => [],
	},
)

const emit = defineEmits<{
	openSettings: []
	dismissSettingsHint: []
}>()

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { serverId } = injectModrinthServerContext()
const isNuxt = computed(() => client instanceof NuxtModrinthClient)
const {
	isInstalling,
	isStopping,
	showRestartButton,
	busyTooltip,
	canTakeAction,
	canKill,
	primaryActionText,
	initiateAction,
	handlePrimaryAction,
} = useServerPowerAction({
	disabled: computed(() => props.powerDisabled),
})

const userPreferences = useStorage(`pyro-server-${serverId}-preferences`, {
	hideSubdomainLabel: false,
})

const headerImage = computed(() => {
	if (props.server?.is_medal) {
		return 'https://cdn-raw.modrinth.com/medal_icon.webp'
	}
	return props.serverImage ?? undefined
})

const leadingItems = computed(() => [
	{
		id: 'server-icon',
		type: 'component' as const,
		component: ServerIcon,
		componentProps: {
			image: headerImage.value,
		},
		class: isNuxt.value ? 'size-15 !rounded-2xl' : 'size-15 !rounded-xl',
	},
])

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

const serverAddress = computed(() => {
	const domain = props.server?.net?.domain
	if (domain) return `${domain}.modrinth.gg`

	const ip = props.server?.net?.ip
	if (!ip) return null
	const port = props.server?.net?.port
	return port ? `${ip}:${port}` : ip
})

const showAddress = computed(
	() =>
		!!serverAddress.value &&
		(!props.server?.net?.domain || !userPreferences.value.hideSubdomainLabel),
)

const metadataItems = computed<HeaderMetadataItem[]>(() => {
	if (props.server?.flows?.intro) {
		return [
			{
				id: 'configuring',
				label: 'Configuring server...',
				icon: SettingsIcon,
			},
		]
	}

	const items: HeaderMetadataItem[] = []
	const worldName = props.activeWorldName
	if (worldName) {
		items.push({
			id: 'world',
			label: worldName,
			icon: GlobeIcon,
		})
	}
	if (props.server?.loader) {
		items.push({
			id: 'loader',
			label: props.server.mc_version
				? `${formatLoaderLabel(props.server.loader)} ${props.server.mc_version}`
				: formatLoaderLabel(props.server.loader),
			icon: LoaderIcon,
			iconProps: {
				loader: props.server.loader,
			},
		})
	}
	if (showAddress.value && serverAddress.value) {
		items.push({
			id: 'address',
			label: serverAddress.value,
			icon: LinkIcon,
			tooltip: 'Copy server address',
			onClick: copyServerAddress,
		})
	}
	if (showUptime.value) {
		items.push({
			id: 'uptime',
			label: formattedUptime.value,
			icon: TimerIcon,
		})
	}
	if (props.serverProject) {
		items.push({
			id: 'project',
			label: `Linked to ${props.serverProject.title}`,
			icon: Avatar,
			iconProps: {
				src: props.serverProject.icon_url ?? undefined,
				alt: props.serverProject.title,
				size: '24px',
			},
			to: serverProjectLink.value,
			class: 'text-primary',
		})
	}
	return items
})

const serverProjectLink = computed(() => {
	if (props.serverProjectLink) {
		return props.serverProjectLink
	}
	if (!props.serverProject) {
		return ''
	}
	return `/project/${props.serverProject.slug ?? props.serverProject.id}`
})

const startActionText = computed(() =>
	primaryActionText.value === 'Start' ? 'Start server' : primaryActionText.value,
)

const powerActionWorlds = computed(() => (props.worlds.length > 1 ? props.worlds : []))

const startSplitActions = computed<JoinedButtonAction[]>(() => [
	{
		id: 'start',
		label: startActionText.value,
		icon: PlayIcon,
		action: handlePrimaryAction,
	},
	...powerActionWorlds.value.map((world) => ({
		id: `start-${world.id}`,
		label: `Start with ${world.name}`,
		icon: GlobeIcon,
		action: () => initiateAction('Start', world.id),
	})),
])

const restartSplitActions = computed<JoinedButtonAction[]>(() => [
	{
		id: 'restart',
		label: primaryActionText.value,
		icon: UpdatedIcon,
		action: () => initiateAction('Restart'),
	},
	// TODO: Implement world scoping when Archon/Kyros support target worlds in power requests.
	...powerActionWorlds.value.map((world) => ({
		id: `restart-${world.id}`,
		label: `Restart with ${world.name}`,
		icon: GlobeIcon,
		action: () => initiateAction('Restart'),
	})),
])

const stopSplitActions = computed<JoinedButtonAction[]>(() => [
	{
		id: 'stop',
		label: isStopping.value ? 'Stopping' : 'Stop',
		icon: StopCircleIcon,
		action: () => initiateAction('Stop'),
	},
	{
		id: 'kill_server',
		label: 'Kill server',
		icon: SlashIcon,
		action: () => initiateAction('Kill'),
	},
])

const powerActions = computed<HeaderAction[]>(() => {
	if (isInstalling.value) {
		return [
			{
				id: 'installing',
				label: 'Installing...',
				icon: LoaderCircleIcon,
				iconClass: 'animate-spin',
				color: 'brand',
				disabled: true,
			},
		]
	}
	if (showRestartButton.value) {
		return [
			{
				id: 'restart',
				label: primaryActionText.value,
				color: 'orange',
				joinedActions: restartSplitActions.value,
				primaryDisabled: !canTakeAction.value,
				dropdownDisabled: !canTakeAction.value,
			},
			{
				id: 'stop',
				label: 'Stop server',
				color: 'red',
				joinedActions: stopSplitActions.value,
				primaryDisabled: !canTakeAction.value,
				dropdownDisabled: !canKill.value,
			},
		]
	}
	if (isStopping.value) {
		return [
			{
				id: 'stop',
				label: 'Stop server',
				color: 'red',
				joinedActions: stopSplitActions.value,
				primaryDisabled: true,
				dropdownDisabled: !canKill.value,
				primaryMuted: true,
			},
		]
	}
	return [
		{
			id: 'start',
			label: startActionText.value,
			color: 'brand',
			tooltip: busyTooltip.value,
			joinedActions: startSplitActions.value,
			primaryDisabled: !canTakeAction.value,
			dropdownDisabled: !canTakeAction.value,
		},
	]
})

const settingsAction = computed<HeaderAction>(() => ({
	id: 'settings',
	label: props.settingsLabel,
	icon: SettingsIcon,
	labelHidden: true,
	tooltip: props.showSettingsHint ? undefined : props.settingsLabel,
	onClick: () => emit('openSettings'),
	prompt: {
		title: props.settingsHintTitle,
		description: props.settingsHintDescription,
		dismissLabel: props.settingsHintDismissLabel,
		shown: props.showSettingsHint,
		placement: 'bottom-end',
		onDismiss: () => emit('dismissSettingsHint'),
	},
}))

const headerActions = computed<HeaderAction[]>(() => [
	...powerActions.value,
	settingsAction.value,
	...props.actions,
])

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
