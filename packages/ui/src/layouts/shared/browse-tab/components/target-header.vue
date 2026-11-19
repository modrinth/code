<script setup lang="ts">
import { NuxtModrinthClient } from '@modrinth/api-client'
import { LeftArrowIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import Admonition from '#ui/components/base/Admonition.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import { IconButton } from '#ui/components/base/buttons'
import PageHeader from '#ui/components/base/page-header/index.vue'
import PageHeaderMetadata from '#ui/components/base/page-header/metadata/index.vue'
import PageHeaderMetadataItem from '#ui/components/base/page-header/metadata/page-header-metadata-item.vue'
import TagIcon from '#ui/components/base/TagIcon.vue'
import ServerIcon from '#ui/components/servers/icons/ServerIcon.vue'
import ServerHeaderMetadataItems from '#ui/components/servers/server-header/server-header-metadata-items.vue'
import {
	usePreviousRoute,
	useRouteEnterTransition,
	useRouteExitTransition,
} from '#ui/composables/route-transitions'
import { useServerIcon } from '#ui/composables/use-server-icon'
import { injectModrinthClient } from '#ui/providers'
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

const router = useRouter()
const client = injectModrinthClient()
const isNuxt = client instanceof NuxtModrinthClient
const leaveModal = ref<InstanceType<typeof SelectedProjectsLeaveModal>>()

const backPath = computed(() => router.resolve(props.target.back.to).path)
const enteringFromBackTarget = useRouteEnterTransition(
	usePreviousRoute()?.path === backPath.value,
)
const leavingToBackTarget = useRouteExitTransition((to) => to.path === backPath.value, 200)
const transitionHidden = computed(() => enteringFromBackTarget.value || leavingToBackTarget.value)

const serverId = computed(() => props.target.serverId ?? '')
const { config: serverIconConfig, image: serverIconImage } = useServerIcon(serverId, {
	enabled: computed(() => !!props.target.serverId),
})

const serverIconSrc = computed(() =>
	props.target.isMedal
		? MEDAL_ICON_URL
		: (serverIconImage.value ?? props.target.iconSrc ?? undefined),
)

const loaderLabel = computed(() =>
	[formatLoaderLabel(props.target.loader ?? ''), props.target.gameVersion]
		.filter(Boolean)
		.join(' '),
)

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
	>
		<template #leading>
			<div
				class="browse-back-button"
				:class="{ 'browse-back-button--hidden': transitionHidden }"
			>
				<IconButton
					v-tooltip="target.back.label"
					size="xl"
					:label="target.back.label"
					native-type="button"
					class="absolute right-0 top-0"
					@click="handleBack"
				>
					<LeftArrowIcon />
				</IconButton>
			</div>
			<ServerIcon
				v-if="target.serverId"
				:image="serverIconSrc"
				:config="target.isMedal ? null : serverIconConfig"
				:class="isNuxt ? 'size-20 !rounded-2xl' : 'size-16 !rounded-xl'"
			/>
			<Avatar
				v-else
				:src="target.iconSrc"
				:alt="target.name"
				size="64px"
				:tint-by="target.tintBy"
				pad-transparent-corners
			/>
		</template>

		<template #metadata>
			<PageHeaderMetadata>
				<slot name="metadata">
					<ServerHeaderMetadataItems
						v-if="target.serverId"
						:server-id="target.serverId"
						:loader="target.loader"
						:game-version="target.gameVersion"
					/>
					<PageHeaderMetadataItem
						v-else-if="target.loader"
						:icon="TagIcon"
						:icon-props="{ tag: formatLoaderLabel(target.loader), enforceType: 'loader' }"
					>
						{{ loaderLabel }}
					</PageHeaderMetadataItem>
				</slot>
				<PageHeaderMetadataItem
					v-if="target.heading"
					class="transition-opacity duration-200 motion-reduce:transition-none"
					:class="{ 'opacity-0': transitionHidden }"
				>
					{{ target.heading }}
				</PageHeaderMetadataItem>
			</PageHeaderMetadata>
		</template>
	</PageHeader>
	<Admonition v-if="target.warning" type="warning" class="mb-1 mt-4">
		{{ target.warning }}
	</Admonition>
</template>

<style scoped>
.browse-back-button {
	position: relative;
	flex-shrink: 0;
	width: 3rem;
	height: 3rem;
	transition:
		width 200ms cubic-bezier(0.4, 0, 0.2, 1),
		margin-right 200ms cubic-bezier(0.4, 0, 0.2, 1),
		opacity 200ms ease;
}

.browse-back-button--hidden {
	width: 0;
	margin-right: -1rem;
	opacity: 0;
	pointer-events: none;
}

@media (prefers-reduced-motion: reduce) {
	.browse-back-button {
		transition: none;
	}
}
</style>
