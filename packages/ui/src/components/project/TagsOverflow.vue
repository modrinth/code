<script setup lang="ts">
import TagItem from '../base/TagItem.vue'
import TagTagItem from '../base/TagTagItem.vue'
import Tooltip from '../floating/Tooltip.vue'

defineProps<{
	tags: string[]
}>()

defineOptions({
	inheritAttrs: false,
})
</script>

<template>
	<Tooltip placement="bottom" panel-class="!rounded-2xl !p-2">
		<TagItem v-if="tags.length > 0" v-bind="$attrs" tabindex="0"> +{{ tags.length }} </TagItem>
		<template #popper>
			<div class="flex gap-1 flex-wrap max-w-[20rem]">
				<TagTagItem
					v-for="tag in tags"
					:key="'overflow-tag-' + tag"
					hide-non-loader-icon
					:tag="tag"
				/>
			</div>
		</template>
	</Tooltip>
</template>
