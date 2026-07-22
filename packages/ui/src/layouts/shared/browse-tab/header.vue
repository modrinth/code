<script setup lang="ts">
import { LeftArrowIcon, TagCategoryGamepad2Icon as Gamepad2Icon } from '@modrinth/assets'
import type { Component } from 'vue'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import Admonition from '#ui/components/base/Admonition.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import PageHeader from '#ui/components/base/page-header/index.vue'
import PageHeaderMetadata from '#ui/components/base/page-header/metadata/index.vue'
import PageHeaderMetadataItem from '#ui/components/base/page-header/metadata/page-header-metadata-item.vue'
import LoaderIcon from '#ui/components/servers/icons/LoaderIcon.vue'
import { useServerImage } from '#ui/composables/use-server-image'
import { formatLoaderLabel } from '#ui/utils/loaders'

import SelectedProjectsLeaveModal from './components/SelectedProjectsLeaveModal.vue'
import { injectBrowseManager } from './providers/browse-manager'
import type { BrowseInstallContext } from './types'

const MEDAL_ICON_URL = 'https://cdn-raw.modrinth.com/medal_icon.webp'

const router = useRouter()
const props = defineProps<{
	installContext?: BrowseInstallContext | null
	divider?: boolean
	bottomPadding?: boolean
}>()
type SelectedProjectsLeaveResult = 'cancel' | 'discard' | 'install'
type BrowseHeaderMetadataItem = {
	id: string
	label: string
	icon?: Component
	iconProps?: Record<string, unknown>
	class?: string
}

const ctx = injectBrowseManager(null)
const installContext = computed(() => props.installContext ?? ctx?.installContext?.value ?? null)
const selectedProjectsLeaveModal = ref<InstanceType<typeof SelectedProjectsLeaveModal>>()

const serverId = computed(() => installContext.value?.serverId ?? '')
const upstream = computed(() => installContext.value?.upstream ?? null)

const { image: fetchedIcon } = useServerImage(serverId, upstream, {
	enabled: computed(() => !!installContext.value?.serverId),
})

const iconSrc = computed(() => {
	if (installContext.value?.isMedal) return MEDAL_ICON_URL
	return fetchedIcon.value ?? installContext.value?.iconSrc ?? null
})

const metadataItems = computed(() => {
	const context = installContext.value
	if (!context) return []

	const items: BrowseHeaderMetadataItem[] = []
	if (context.heading) {
		items.push({
			id: 'heading',
			label: context.heading,
			class: '!text-primary',
		})
	}
	if (context.gameVersion) {
		items.push({
			id: 'game-version',
			label: `Minecraft ${context.gameVersion}`,
			icon: Gamepad2Icon,
			class: '!text-primary',
		})
	}
	if (context.loader) {
		const loaderName = formatLoaderLabel(context.loader)
		const loaderLabel = [loaderName, context.loaderVersion].filter(Boolean).join(' ')
		items.push({
			id: 'loader',
			label: loaderLabel,
			icon: LoaderIcon,
			iconProps: { loader: loaderName },
			class: '!text-primary',
		})
	}
	return items
})

const selectedCount = computed(() => installContext.value?.selectedProjects?.length ?? 0)
const isInstallingSelected = computed(() => installContext.value?.isInstallingSelected ?? false)

async function handleBack() {
	const context = installContext.value
	if (!context) return

	if (selectedCount.value > 0 && !isInstallingSelected.value) {
		if (context.skipNonEssentialWarnings) {
			await handleSelectedProjectsLeaveResult('discard', context)
			return
		}

		const result = await selectedProjectsLeaveModal.value?.prompt()
		await handleSelectedProjectsLeaveResult(result ?? 'cancel', context)
		return
	}

	const shouldNavigate = await context.onBack?.()
	if (shouldNavigate === false) return

	await router.push(context.backUrl)
}

async function handleSelectedProjectsLeaveResult(
	result: SelectedProjectsLeaveResult,
	context: BrowseInstallContext,
) {
	if (result === 'cancel') return
	if (result === 'install') {
		const shouldNavigate = await context.installSelected?.()
		if (shouldNavigate === false) return
		return
	}

	if (context.discardSelectedAndBack) {
		await context.discardSelectedAndBack()
		return
	}

	await (context.clearSelected ?? context.clearQueued)?.()
	await router.push(context.backUrl)
}
</script>

<template>
	<template v-if="installContext">
		<SelectedProjectsLeaveModal
			ref="selectedProjectsLeaveModal"
			:count="selectedCount"
			:installing="isInstallingSelected"
		/>
		<PageHeader
			:title="installContext.name"
			:divider="props.divider ?? false"
			:bottom-padding="props.bottomPadding ?? false"
			main-class="items-center"
			title-class="leading-8"
			truncate-title
		>
			<template #leading>
				<ButtonStyled circular size="large">
					<button
						v-tooltip="installContext.backLabel"
						type="button"
						:aria-label="installContext.backLabel"
						@click="handleBack"
					>
						<LeftArrowIcon />
					</button>
				</ButtonStyled>
				<Avatar
					v-if="iconSrc"
					:src="iconSrc"
					:alt="installContext.name"
					size="48px"
					class="shrink-0"
				/>
			</template>

			<template v-if="metadataItems.length" #metadata>
				<PageHeaderMetadata>
					<PageHeaderMetadataItem
						v-for="item in metadataItems"
						:key="item.id"
						:icon="item.icon"
						:icon-props="item.iconProps"
						:class="item.class"
					>
						{{ item.label }}
					</PageHeaderMetadataItem>
				</PageHeaderMetadata>
			</template>
		</PageHeader>
		<Admonition v-if="installContext.warning" type="warning" class="mb-1">
			{{ installContext.warning }}
		</Admonition>
	</template>
</template>
