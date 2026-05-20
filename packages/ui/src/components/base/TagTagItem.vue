<template>
	<TagItem :action="action" :style="isLoader ? `--_color: var(--color-platform-${tag})` : ''">
		<component :is="icon" v-if="icon" />
		<FormattedTag :tag="tag" />
	</TagItem>
</template>
<script setup lang="ts">
import { getTagIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { getTagMessage } from '../../utils'
import FormattedTag from './FormattedTag.vue'
import TagItem from './TagItem.vue'

const props = withDefaults(
	defineProps<{
		tag: string
		hideNonLoaderIcon?: boolean
		action?: (event: MouseEvent) => void
	}>(),
	{
		hideNonLoaderIcon: false,
		action: undefined,
	},
)

const icon = computed(() =>
	props.hideNonLoaderIcon && !isLoader.value ? undefined : getTagIcon(props.tag),
)
const isLoader = computed(() => getTagMessage(props.tag, 'loader') !== undefined)
</script>
