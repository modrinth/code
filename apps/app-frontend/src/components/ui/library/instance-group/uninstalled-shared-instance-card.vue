<script setup lang="ts">
import { DownloadIcon, XIcon } from '@modrinth/assets'
import { Button, IconButton, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import InstanceCardView from '@/components/ui/library/instance-group/instance-card-view.vue'
import InstanceSelectionButton from '@/components/ui/library/instance-group/instance-selection-button.vue'
import InstanceSharedBadge from '@/components/ui/library/instance-group/instance-shared-badge.vue'
import { getLibraryInstanceSelectionKey, useLibrary } from '@/components/ui/library/use-library'
import { useHiddenInstances } from '@/features/library/hidden-instances/use-hidden-instances'
import { libraryInstanceMessages } from '@/features/library/messages'
import { injectSharedInstanceActions } from '@/features/library/shared-instances/actions-context'
import type { UninstalledSharedInstance } from '@/features/library/shared-instances/types'

const props = defineProps<{
	instance: UninstalledSharedInstance
	instanceGroupId: string
	isSelectionAnchor?: boolean
}>()

const emit = defineEmits<{
	(e: 'toggle-selection', shiftKey: boolean): void
}>()

const { formatMessage } = useVIntl()
const {
	selectedLibraryInstances,
	isLibraryInstanceSelectionActive,
	canSelectLibraryInstance,
	getSharedInstanceOwner,
	handleUninstalledSharedInstanceContextMenu,
} = useLibrary()
const { install, prefetchInstall } = injectSharedInstanceActions()
const { hide } = useHiddenInstances()

const selected = computed(() =>
	selectedLibraryInstances.value.has(
		getLibraryInstanceSelectionKey({
			instanceId: props.instance.id,
			groupId: props.instanceGroupId,
		}),
	),
)

const selectable = computed(() => selected.value || canSelectLibraryInstance(props.instance.id))
const owner = computed(() => getSharedInstanceOwner(props.instance))
const isOwner = computed(() => props.instance.role === 'owner')

const toggleSelection = (event?: MouseEvent) => {
	emit('toggle-selection', event?.shiftKey ?? false)
}

const activateCard = (event: MouseEvent) => {
	if (isLibraryInstanceSelectionActive.value || event.shiftKey) {
		if (selectable.value) toggleSelection(event)
	} else {
		install(props.instance, { event })
	}
}

const handleKeydown = (event: KeyboardEvent) => {
	if (event.target !== event.currentTarget) return

	if (event.key === 'Enter') {
		event.preventDefault()
		if (isLibraryInstanceSelectionActive.value) {
			toggleSelection()
		} else {
			install(props.instance)
		}
	} else if (event.key === ' ' && isLibraryInstanceSelectionActive.value) {
		event.preventDefault()
		toggleSelection()
	}
}
</script>

<template>
	<InstanceCardView
		class="group/card cursor-pointer -outline-offset-2 focus-visible:!outline-2 hover:brightness-110"
		:instance="instance"
		:selected="selected"
		dimmed
		:class="{ '!cursor-default opacity-50': !selectable }"
		data-library-instance-card
		:data-instance-id="instance.id"
		:data-instance-group="instanceGroupId"
		role="button"
		tabindex="0"
		:aria-label="
			isLibraryInstanceSelectionActive
				? formatMessage(
						selected
							? libraryInstanceMessages.deselectWithName
							: libraryInstanceMessages.selectWithName,
						{
							name: instance.name,
						},
					)
				: formatMessage(libraryInstanceMessages.installWithName, { name: instance.name })
		"
		:aria-pressed="isLibraryInstanceSelectionActive ? selected : undefined"
		@click="activateCard"
		@keydown="handleKeydown"
		@mouseenter="prefetchInstall(instance)"
		@focus="prefetchInstall(instance)"
		@contextmenu.prevent.stop="
			(event: MouseEvent) => handleUninstalledSharedInstanceContextMenu(event, instance)
		"
	>
		<template #icon-overlay="{ compact }">
			<InstanceSharedBadge
				v-if="isOwner || owner"
				:owner="owner"
				:is-owner="isOwner"
				:compact="compact"
			/>
			<div
				v-if="!compact && !isLibraryInstanceSelectionActive"
				class="absolute bottom-2 right-2 z-[2] flex origin-bottom-right scale-90 items-center gap-1.5 opacity-0 transition-[opacity,transform] group-hover/card:scale-100 group-hover/card:opacity-100 group-focus-visible/card:scale-100 group-focus-visible/card:opacity-100 group-has-[:focus-visible]/card:scale-100 group-has-[:focus-visible]/card:opacity-100"
			>
				<IconButton
					v-tooltip="formatMessage(libraryInstanceMessages.hideFromLibrary)"
					:label="formatMessage(libraryInstanceMessages.hideFromLibrary)"
					size="sm"
					class="border border-solid border-surface-5 !bg-surface-4"
					@click.stop="hide(instance.id)"
				>
					<XIcon />
				</IconButton>
				<Button
					type="colored"
					color="brand"
					size="sm"
					class="!rounded-xl !px-2.5"
					@click.stop="(event: MouseEvent) => install(instance, { event })"
				>
					<DownloadIcon />
					{{ formatMessage(libraryInstanceMessages.install) }}
				</Button>
			</div>
		</template>
		<template #leading="{ compact }">
			<IconButton
				v-if="compact && !isLibraryInstanceSelectionActive"
				v-tooltip="formatMessage(libraryInstanceMessages.install)"
				:label="formatMessage(libraryInstanceMessages.install)"
				type="colored"
				color="brand"
				size="md"
				class="origin-bottom scale-75 opacity-0 transition-opacity group-hover/card:scale-100 group-hover/card:opacity-100 group-focus-visible/card:scale-100 group-focus-visible/card:opacity-100 group-has-[:focus-visible]/card:scale-100 group-has-[:focus-visible]/card:opacity-100"
				@click.stop="(event: MouseEvent) => install(instance, { event })"
			>
				<DownloadIcon />
			</IconButton>
		</template>
		<template #overlay="{ compact }">
			<InstanceSelectionButton
				v-if="selectable"
				:selected="selected"
				:compact="compact"
				:is-selection-anchor="isSelectionAnchor"
				@toggle="toggleSelection"
			/>
		</template>
	</InstanceCardView>
</template>
