<template>
	<TagItem :action="action" :style="isLoader ? `--_color: var(--color-platform-${tag})` : ''">
		<FormattedTag :tag="tag" />
	</TagItem>
</template>
<script setup lang="ts">
import { computed } from 'vue'

import { getTagMessage } from '../../utils'
import FormattedTag from './FormattedTag.vue'
import TagItem from './TagItem.vue'

const props = withDefaults(
	defineProps<{
		tag: string
		action?: (event: MouseEvent) => void
	}>(),
	{
		action: undefined,
	},
)

const isLoader = computed(() => getTagMessage(props.tag, 'loader') !== undefined)
</script>
