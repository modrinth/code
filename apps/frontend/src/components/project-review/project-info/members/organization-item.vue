<template>
	<div class="flex min-w-0 items-start gap-2">
		<button
			v-if="iconUrl"
			type="button"
			class="shrink-0 cursor-zoom-in rounded-full border-0 bg-transparent p-0"
			:aria-label="
				formatMessage(messages.openOrganizationIcon, { organization: organization.name })
			"
			@click="viewer?.show(0)"
		>
			<Avatar
				:src="organization.icon_url || iconUrl"
				:alt="organization.name"
				:tint-by="organization.id"
				size="2rem"
				no-shadow
			/>
		</button>
		<Avatar v-else :alt="organization.name" :tint-by="organization.id" size="2rem" no-shadow />
		<ImageViewerEditor
			:key="organization.id"
			ref="viewer"
			:items="viewerItems"
			editor="disabled"
			:pixelated="pixelated"
		>
			<template #actions="{ item }">
				<ImageViewerActions v-model:pixelated="pixelated" :src="item.src" />
			</template>
		</ImageViewerEditor>
		<div class="min-w-0">
			<div class="flex items-center gap-1.5">
				<NuxtLink
					:to="`/organization/${organization.slug}`"
					target="_blank"
					class="truncate font-medium text-primary hover:underline"
					>{{ organization.name }}</NuxtLink
				>
				<CrownIcon class="size-3 shrink-0" :aria-label="formatMessage(messages.owner)" />
			</div>
			<div class="flex items-center gap-1 text-xs">
				<p class="m-0 text-xs">{{ formatMessage(messages.organization) }}</p>
				<span aria-hidden="true">•</span>
				<p class="m-0">{{ formatMessage(messages.memberCount, { count: memberCount }) }}</p>
			</div>
			<ProjectStatusStats :stats="stats" />
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CrownIcon } from '@modrinth/assets'
import { Avatar, ImageViewerEditor, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import ProjectStatusStats from '~/components/ProjectStatusStats.vue'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import ImageViewerActions from '../../image-viewer-actions.vue'
import { projectReviewMessages as messages } from '../../messages'

const props = defineProps<{
	organization: Labrinth.Projects.v3.Organization
	stats: { status: string; count: number }[]
}>()
const { formatMessage } = useVIntl()
const viewer = ref<InstanceType<typeof ImageViewerEditor>>()
const { pixelated } = injectProjectReviewPageContext()
const memberCount = computed(
	() => props.organization.members.filter((member) => member.accepted).length,
)
const iconUrl = computed(() => props.organization.raw_icon_url || props.organization.icon_url)
const viewerItems = computed(() =>
	iconUrl.value ? [{ id: iconUrl.value, src: iconUrl.value, alt: props.organization.name }] : [],
)
</script>
