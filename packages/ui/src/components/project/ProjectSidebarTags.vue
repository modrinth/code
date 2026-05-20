<template>
	<div v-if="allTags.length > 0" class="flex flex-col gap-3">
		<h2 class="text-lg m-0">{{ formatMessage(messages.title) }}</h2>
		<div class="flex flex-wrap gap-1">
			<TagItem
				v-for="tag in allTags"
				:key="tag"
				:action="props.project.actualProjectType ? () => handleClickTag(tag) : undefined"
			>
				<FormattedTag :tag="tag" />
			</TagItem>
		</div>
	</div>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'

import { defineMessages, useVIntl } from '../../composables'
import FormattedTag from '../base/FormattedTag.vue'
import TagItem from '../base/TagItem.vue'

const router = useRouter()

const handleClickTag = (tag: string) => {
	if (!props.project.actualProjectType) return

	const projectType =
		props.project.actualProjectType === 'minecraft_java_server'
			? 'server'
			: props.project.actualProjectType

	const params = projectType === 'server' ? `sc=${tag}` : `f=categories:${tag}`

	router.push(`/discover/${projectType}?${params}`)
}

const props = defineProps<{
	project: {
		categories: string[]
		additional_categories: string[]
		actualProjectType?: string
	}
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'project.about.tags.title',
		defaultMessage: 'Tags',
	},
})

const allTags = computed(() => [
	...props.project.categories,
	...props.project.additional_categories,
])
</script>
