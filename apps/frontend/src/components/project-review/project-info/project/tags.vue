<template>
	<Section :heading="formatMessage(messages.tags)" :target="{ kind: 'tags' }">
		<template #actions>
			<EditButton :section="formatMessage(messages.tags)" @click="editModal?.show()" />
		</template>
		<EditModal :key="project?.id" ref="editModal" />
		<div v-if="project && hasTags" class="flex flex-wrap gap-1.5">
			<TagItem
				v-for="category in project.categories"
				:key="category"
				class="!border-brand !bg-transparent !text-brand opacity-90"
				>{{ formatCategory(formatMessage, category) }}</TagItem
			>
			<TagItem
				v-for="category in project.additional_categories"
				:key="category"
				class="!border-surface-4 !bg-surface-3 !text-secondary"
			>
				{{ formatCategory(formatMessage, category) }}
			</TagItem>
		</div>
		<span v-else>{{ formatMessage(messages.emptyTags) }}</span>
	</Section>
</template>

<script setup lang="ts">
import { formatCategory, TagItem, useVIntl } from '@modrinth/ui'
import { computed, useTemplateRef } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../../messages'
import EditButton from '../edit/button.vue'
import EditModal from '../edit/tags.vue'
import Section from '../section.vue'

const { project } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
const editModal = useTemplateRef<InstanceType<typeof EditModal>>('editModal')
const hasTags = computed(
	() => !!(project.value?.categories.length || project.value?.additional_categories.length),
)
</script>
