<template>
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
		<ImageViewerEditor :key="projectId" ref="viewer" :items="viewerItems" editor="disabled">
			<template #actions="{ item }">
				<ButtonLink
					v-tooltip="formatMessage(messages.openImageInNewTab)"
					type="quiet"
					class="!w-9 !rounded-full !p-0"
					:aria-label="formatMessage(messages.openImageInNewTab)"
					:href="item.src"
					target="_blank"
				>
					<ExternalIcon aria-hidden="true" />
				</ButtonLink>
			</template>
		</ImageViewerEditor>
	</template>
</template>

<script setup lang="ts">
import { ExternalIcon } from '@modrinth/assets'
import { Avatar, ButtonLink, ImageViewerEditor, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../../messages'

const { formatMessage } = useVIntl()
const { project, projectId } = injectProjectReviewPageContext()
const viewer = ref<InstanceType<typeof ImageViewerEditor>>()
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
</script>
