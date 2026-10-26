<script setup lang="ts">
import { DownloadIcon, SpinnerIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import InstanceCardView from '@/components/ui/library/instance-group/instance-card-view.vue'
import InstanceOwnerBadge from '@/components/ui/library/instance-group/instance-owner-badge.vue'
import InstanceSelectionButton from '@/components/ui/library/instance-group/instance-selection-button.vue'
import { getLibraryInstanceSelectionKey, useLibrary } from '@/components/ui/library/use-library'
import type { UninstalledSharedInstance } from '@/components/ui/library/use-uninstalled-shared-instances'

const props = defineProps<{
	instance: UninstalledSharedInstance
	instanceGroupId: string
	isSelectionAnchor?: boolean
}>()

const emit = defineEmits<{
	(e: 'toggle-selection', shiftKey: boolean): void
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	installInstance: {
		id: 'app.library.instance.install-with-name',
		defaultMessage: 'Install {name}',
	},
	notInstalled: {
		id: 'app.library.instance.not-installed',
		defaultMessage: 'Not installed',
	},
	selectInstance: {
		id: 'app.library.instance.select-with-name',
		defaultMessage: 'Select {name}',
	},
	deselectInstance: {
		id: 'app.library.instance.deselect-with-name',
		defaultMessage: 'Deselect {name}',
	},
})
const {
	selectedLibraryInstances,
	isLibraryInstanceSelectionActive,
	canSelectLibraryInstance,
	getSharedInstanceOwner,
	loadingSharedInstanceIds,
	installUninstalledSharedInstance,
	handleUninstalledSharedInstanceContextMenu,
} = useLibrary()

const loading = computed(() =>
	loadingSharedInstanceIds.value.has(props.instance.shared_instance_id),
)

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

const toggleSelection = (event?: MouseEvent) => {
	emit('toggle-selection', event?.shiftKey ?? false)
}

const activateCard = (event: MouseEvent) => {
	if (isLibraryInstanceSelectionActive.value || event.shiftKey) {
		if (selectable.value) toggleSelection(event)
	} else {
		void installUninstalledSharedInstance(props.instance, event)
	}
}

const handleKeydown = (event: KeyboardEvent) => {
	if (event.target !== event.currentTarget) return

	if (event.key === 'Enter') {
		event.preventDefault()
		if (isLibraryInstanceSelectionActive.value) {
			toggleSelection()
		} else {
			void installUninstalledSharedInstance(props.instance)
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
				? formatMessage(selected ? messages.deselectInstance : messages.selectInstance, {
						name: instance.name,
					})
				: formatMessage(messages.installInstance, { name: instance.name })
		"
		:aria-pressed="isLibraryInstanceSelectionActive ? selected : undefined"
		@click="activateCard"
		@keydown="handleKeydown"
		@contextmenu.prevent.stop="
			(event: MouseEvent) => handleUninstalledSharedInstanceContextMenu(event, instance)
		"
	>
		<template #icon-overlay="{ compact }">
			<div class="pointer-events-none absolute inset-0 z-[1] flex items-center justify-center">
				<SpinnerIcon
					v-if="loading"
					class="relative animate-spin text-contrast"
					:class="compact ? 'size-5' : 'size-[30%]'"
				/>
				<DownloadIcon
					v-else
					v-tooltip="formatMessage(messages.notInstalled)"
					class="pointer-events-auto relative text-contrast"
					:class="compact ? 'size-5' : 'size-[30%]'"
				/>
			</div>
			<InstanceOwnerBadge v-if="owner" :owner="owner" :compact="compact" />
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
