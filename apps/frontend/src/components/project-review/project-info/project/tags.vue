<template>
	<Section heading="Tags" :target="{ kind: 'tags' }">
		<template #right>
			<EditButton section="Tags" @click="editModal?.show()" />
		</template>
		<EditModal :key="project?.id" ref="editModal" />
		<div v-if="project && hasTags" class="flex flex-wrap gap-1.5">
			<TagItem
				v-for="category in project.categories"
				:key="category"
				class="!border-brand !bg-transparent !text-brand opacity-90"
				>{{ categoryName(category) }}</TagItem
			>
			<TagItem
				v-for="category in project.additional_categories"
				:key="category"
				class="!border-surface-4 !bg-surface-3 !text-secondary"
			>
				{{ categoryName(category) }}
			</TagItem>
		</div>
		<span v-else>No tags</span>
	</Section>
</template>

<script setup lang="ts">
import { TagItem } from '@modrinth/ui'
import { computed, useTemplateRef } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import EditButton from '../edit/button.vue'
import EditModal from '../edit/tags.vue'
import Section from '../section.vue'
import { categoryName } from '../tag-labels'

const { project } = injectProjectReviewPageContext()

const editModal = useTemplateRef<InstanceType<typeof EditModal>>('editModal')
const hasTags = computed(
	() => !!(project.value?.categories.length || project.value?.additional_categories.length),
)
</script>
