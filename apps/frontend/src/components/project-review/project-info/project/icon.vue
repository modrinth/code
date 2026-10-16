<template>
	<Section :heading="formatMessage(messages.icon)" :target="{ kind: 'icon' }">
		<template v-if="iconUrl" #heading-after>
			<KbdChip keybind="review-toggle-icon" />
		</template>
		<template v-if="project">
			<button
				v-if="iconUrl"
				type="button"
				class="w-fit cursor-zoom-in rounded-xl border-0 bg-transparent p-0"
				:aria-label="formatMessage(messages.openIcon)"
				@click="viewer?.show(0)"
			>
				<Avatar :src="project.icon_url || iconUrl" :alt="project.name" size="5rem" no-shadow />
			</button>
			<Avatar v-else :alt="project.name" size="5rem" no-shadow />
			<ImageViewerEditor
				:key="projectId"
				ref="viewer"
				:items="viewerItems"
				editor="disabled"
				:pixelated="pixelated"
				@show="viewerOpen = true"
				@hide="viewerOpen = false"
			>
				<template #actions="{ item, metadata }">
					<ImageViewerActions v-model:pixelated="pixelated" :src="item.src" :metadata="metadata" />
				</template>
			</ImageViewerEditor>
		</template>
	</Section>
</template>

<script setup lang="ts">
import { Avatar, ImageViewerEditor, useVIntl } from '@modrinth/ui'
import { useEventListener } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import { useModerationKeybinds } from '~/composables/moderation'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import ImageViewerActions from '../../image-viewer-actions.vue'
import KbdChip from '../../kdb-chip.vue'
import { projectReviewMessages as messages } from '../../messages'
import { useReviewShortcut } from '../../shortcuts'
import Section from '../section.vue'

const { formatMessage } = useVIntl()
const { project, projectId, pixelated } = injectProjectReviewPageContext()
const viewer = ref<InstanceType<typeof ImageViewerEditor>>()
const viewerOpen = ref(false)
const keybinds = useModerationKeybinds()
const iconUrl = computed(() => project.value?.raw_icon_url || project.value?.icon_url)
const viewerItems = computed(() => {
	if (!project.value || !iconUrl.value) return []
	return [
		{
			id: iconUrl.value,
			src: iconUrl.value,
			alt: project.value.name,
		},
	]
})

watch(projectId, () => {
	viewerOpen.value = false
})

function toggleViewer() {
	if (viewerOpen.value) viewer.value?.hide()
	else viewer.value?.show(0)
}

useReviewShortcut('toggle-icon', toggleViewer, () => !!viewer.value && !!iconUrl.value)

useEventListener('keydown', (event) => {
	if (
		!viewerOpen.value ||
		event.defaultPrevented ||
		event.repeat ||
		event.isComposing ||
		document.querySelector('.modal-root')
	)
		return
	const target = event.target
	if (
		target instanceof HTMLElement &&
		(target.isContentEditable ||
			!!target.closest('input, textarea, select, [role="textbox"], .cm-editor'))
	)
		return
	keybinds.value.handle(event, {
		scope: 'review-actions',
		available: (action) => action === 'toggle-icon',
		run: toggleViewer,
	})
})
</script>
