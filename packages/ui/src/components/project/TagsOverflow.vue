<script setup lang="ts">
import { Menu } from 'floating-vue'

import { TagItem, TagTagItem } from '../base'

defineProps<{
	tags: string[]
}>()

defineOptions({
	inheritAttrs: false,
})
</script>

<template>
	<Menu :delay="{ hide: 50, show: 0 }" no-auto-focus>
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
	</Menu>
</template>
