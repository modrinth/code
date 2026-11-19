<script setup lang="ts">
import { LeftArrowIcon, TagCategoryGamepad2Icon as Gamepad2Icon } from '@modrinth/assets'
import type { Component } from 'vue'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import Admonition from '#ui/components/base/Admonition.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import { IconButton } from '#ui/components/base/buttons'
import PageHeader from '#ui/components/base/page-header/index.vue'
import PageHeaderMetadata from '#ui/components/base/page-header/metadata/index.vue'
import PageHeaderMetadataItem from '#ui/components/base/page-header/metadata/page-header-metadata-item.vue'
import TagIcon from '#ui/components/base/TagIcon.vue'
import { useGeneratedIconSrc } from '#ui/composables/use-generated-icon-src'
import { useServerIcon } from '#ui/composables/use-server-icon'
import { formatLoaderLabel } from '#ui/utils/loaders'

import type { BrowseSelection, BrowseTarget } from '../types'
import SelectedProjectsLeaveModal from './selection/leave-modal.vue'

const MEDAL_ICON_URL = 'https://cdn.modrinth.com/medal_icon.webp'

const props = defineProps<{
	target: BrowseTarget
	selection?: BrowseSelection | null
	skipLeavePrompt?: boolean
	divider?: boolean
	bottomPadding?: boolean
}>()

type HeaderMetadataItem = {
	id: string
	label: string
	icon?: Component
	iconProps?: Record<string, unknown>
}

const router = useRouter()
const leaveModal = ref<InstanceType<typeof SelectedProjectsLeaveModal>>()

const serverId = computed(() => props.target.serverId ?? '')
const { config: serverIconConfig, image: serverIconImage } = useServerIcon(serverId, {
	enabled: computed(() => !!props.target.serverId),
})
const serverIconGeneratedSrc = useGeneratedIconSrc(serverIconConfig)

const iconSrc = computed(() => {
	if (props.target.isMedal) return MEDAL_ICON_URL
	return serverIconGeneratedSrc.value ?? serverIconImage.value ?? props.target.iconSrc ?? null
})

const metadataItems = computed(() => {
	const target = props.target
	const items: HeaderMetadataItem[] = []
	if (target.heading) {
		items.push({ id: 'heading', label: target.heading })
	}
	if (target.gameVersion) {
		items.push({
			id: 'game-version',
			label: `Minecraft ${target.gameVersion}`,
			icon: Gamepad2Icon,
		})
	}
	if (target.loader) {
		const loaderName = formatLoaderLabel(target.loader)
		items.push({
			id: 'loader',
			label: [loaderName, target.loaderVersion].filter(Boolean).join(' '),
			icon: TagIcon,
			iconProps: { tag: loaderName, enforceType: 'loader' },
		})
	}
	return items
})

const selectedCount = computed(() => props.selection?.projects.value.length ?? 0)
const installingSelected = computed(() => props.selection?.installing.value ?? false)

async function handleBack() {
	const selection = props.selection
	if (installingSelected.value) return

	if (selection && selectedCount.value > 0) {
		const result = props.skipLeavePrompt ? 'discard' : await leaveModal.value?.prompt()
		if (result === 'install') await selection.installAndBack()
		else if (result === 'discard') await selection.discardAndBack()
		return
	}

	await router.push(props.target.back.to)
}
</script>

<template>
	<SelectedProjectsLeaveModal
		ref="leaveModal"
		:count="selectedCount"
		:installing="installingSelected"
	/>
	<PageHeader
		:title="target.name"
		:divider="divider ?? false"
		:bottom-padding="bottomPadding ?? false"
		main-class="items-center"
		title-class="leading-8"
		truncate-title
	>
		<template #leading>
			<IconButton
				v-tooltip="target.back.label"
				size="xl"
				:label="target.back.label"
				native-type="button"
				@click="handleBack"
			>
				<LeftArrowIcon />
			</IconButton>
			<Avatar
				v-if="iconSrc"
				:src="iconSrc"
				:alt="target.name"
				size="48px"
				class="shrink-0"
				:pad-transparent-corners="!target.serverId"
			/>
		</template>

		<template v-if="metadataItems.length" #metadata>
			<PageHeaderMetadata>
				<PageHeaderMetadataItem
					v-for="item in metadataItems"
					:key="item.id"
					:icon="item.icon"
					:icon-props="item.iconProps"
					class="!text-primary"
				>
					{{ item.label }}
				</PageHeaderMetadataItem>
			</PageHeaderMetadata>
		</template>
	</PageHeader>
	<Admonition v-if="target.warning" type="warning" class="mb-1 mt-4">
		{{ target.warning }}
	</Admonition>
</template>
