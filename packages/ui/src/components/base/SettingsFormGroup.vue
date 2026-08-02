<script setup lang="ts">
defineProps<{
	title?: string
	titleFor?: string
	description?: string
}>()
</script>

<template>
	<div class="flex flex-col gap-2">
		<div
			v-if="title || description || $slots.title || $slots.description"
			:class="description || $slots.description ? 'flex flex-col gap-1' : undefined"
		>
			<label v-if="title && titleFor" :for="titleFor" class="m-0 leading-normal text-contrast">
				{{ title }}
			</label>
			<p v-else-if="title" class="m-0 leading-normal text-contrast">
				{{ title }}
			</p>
			<slot name="title" />
			<p v-if="description" class="m-0 leading-normal text-primary">
				{{ description }}
			</p>
			<slot name="description" />
		</div>
		<slot />
	</div>
</template>
