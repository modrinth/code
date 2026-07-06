<template>
	<component
		:is="item.icon"
		v-if="item.icon"
		class="flex size-5 shrink-0 text-current"
		aria-hidden="true"
		v-bind="item.iconProps"
	/>
	<Avatar
		v-if="item.avatarSrc"
		:src="item.avatarSrc"
		:alt="item.avatarAlt ?? ''"
		:size="item.avatarSize ?? '24px'"
		:tint-by="item.avatarTintBy ?? null"
		:circle="item.avatarCircle ?? false"
		:no-shadow="item.avatarNoShadow ?? false"
		:raised="item.avatarRaised ?? false"
	/>
	<span v-if="item.label" :class="['truncate', item.labelClass]">
		{{ item.label }}
	</span>
	<span
		v-if="item.value !== undefined && item.value !== null"
		:class="['truncate', item.valueClass]"
	>
		{{ item.value }}
	</span>
	<div v-if="item.tags?.length" class="flex flex-wrap gap-2">
		<TagItem v-for="tag in item.tags" :key="tag.id" :action="tag.onClick">
			<FormattedTag v-if="tag.tag" :tag="tag.tag" />
			<span v-else>{{ tag.label }}</span>
		</TagItem>
	</div>
</template>

<script setup lang="ts">
import Avatar from '../Avatar.vue'
import FormattedTag from '../FormattedTag.vue'
import TagItem from '../TagItem.vue'
import type { PageHeaderMetadataContentItem } from './types'

defineProps<{
	item: PageHeaderMetadataContentItem
}>()
</script>
