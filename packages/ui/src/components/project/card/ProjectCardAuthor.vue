<script setup lang="ts">
import { AutoLink } from '../../base'

export type ProjectCardAuthorDetails = {
	name: string
	link?: string
}

defineProps<{
	author: ProjectCardAuthorDetails
}>()
</script>

<template>
	<!-- author will try to be full width, but no longer than the proper fit. but this also allows it to truncate if necessary -->
	<span class="line-clamp-1 break-all text-secondary font-normal max-w-fit w-full">
		by
		<AutoLink
			:to="author.link"
			:class="
				author.link
					? 'focus:underline focus:brightness-[--hover-brightness] hover:brightness-[--hover-brightness] hover:underline smart-clickable:allow-pointer-events'
					: ''
			"
		>
			{{ author.name }}
		</AutoLink>
	</span>
</template>
