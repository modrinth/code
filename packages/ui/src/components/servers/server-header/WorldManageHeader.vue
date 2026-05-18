<template>
	<div class="w-full flex flex-col gap-4" :class="{ 'mt-4': isNuxt }">
		<PageHeader
			:header="props.name || props.fallbackName"
			:leading="leadingItems"
			:metadata="headerMetadata"
			:actions="props.actions"
			:header-class="props.headerClass"
		/>
	</div>
</template>

<script setup lang="ts">
import { NuxtModrinthClient } from '@modrinth/api-client'
import { LeftArrowIcon, TagCategoryGamepad2Icon as Gamepad2Icon, TimerIcon } from '@modrinth/assets'
import { type Component, computed } from 'vue'
import { useRouter } from 'vue-router'

import PageHeader from '#ui/components/base/PageHeader.vue'
import LoaderIcon from '#ui/components/servers/icons/LoaderIcon.vue'
import { injectModrinthClient } from '#ui/providers'
import { formatLoaderLabel } from '#ui/utils/loaders'

type MetadataItem = {
	id: string
	label: string
	icon: Component
	iconProps?: Record<string, unknown>
}
type HeaderAction = {
	id: string
	label: string
	icon?: Component
	iconProps?: Record<string, unknown>
	iconClass?: string
	tooltip?: string
	ariaLabel?: string
	onClick?: () => void | Promise<void>
	disabled?: boolean
	labelHidden?: boolean
	circular?: boolean
	color?: 'standard' | 'brand' | 'red' | 'orange' | 'green' | 'blue' | 'purple'
	size?: 'standard' | 'large' | 'small'
	type?: 'standard' | 'outlined' | 'transparent' | 'highlight' | 'highlight-colored-text' | 'chip'
	joinedActions?: {
		id: string
		label: string
		icon?: Component
		action: () => void
		color?: 'standard' | 'brand' | 'red' | 'orange' | 'green' | 'blue' | 'purple'
		hoverFilled?: boolean
	}[]
	primaryDisabled?: boolean
	dropdownDisabled?: boolean
	primaryMuted?: boolean
}

const props = withDefaults(
	defineProps<{
		name?: string | null
		metadataItems?: MetadataItem[]
		gameVersion?: string | null
		loader?: string | null
		loaderVersion?: string | null
		lastActive?: string | null
		backHref: string
		backLabel: string
		fallbackName?: string
		headerClass?: string
		actions?: HeaderAction[]
	}>(),
	{
		name: null,
		metadataItems: () => [],
		gameVersion: null,
		loader: null,
		loaderVersion: null,
		lastActive: null,
		fallbackName: 'World',
		headerClass: '',
		actions: () => [],
	},
)

const client = injectModrinthClient()
const router = useRouter()
const isNuxt = computed(() => client instanceof NuxtModrinthClient)
const leadingItems = computed(() => [
	{
		id: 'back',
		type: 'button' as const,
		icon: LeftArrowIcon,
		ariaLabel: props.backLabel,
		tooltip: props.backLabel,
		onClick: () => router.push(props.backHref),
	},
])
const loaderLabel = computed(() => {
	if (!props.loader) return null
	const label = formatLoaderLabel(props.loader.toLowerCase())
	return [label, props.loaderVersion].filter(Boolean).join(' ')
})
const headerMetadata = computed<MetadataItem[]>(() => {
	if (props.metadataItems.length) return props.metadataItems

	const items: MetadataItem[] = []
	if (props.gameVersion) {
		items.push({
			id: 'game-version',
			label: props.gameVersion,
			icon: Gamepad2Icon,
		})
	}
	if (props.loader && loaderLabel.value) {
		items.push({
			id: 'loader',
			label: loaderLabel.value,
			icon: LoaderIcon,
			iconProps: {
				loader: formatLoaderLabel(props.loader.toLowerCase()),
			},
		})
	}
	if (props.lastActive) {
		items.push({
			id: 'last-active',
			label: props.lastActive,
			icon: TimerIcon,
		})
	}

	return items
})
</script>
