<script setup lang="ts">
import { useFormatDateTime } from '../../composables'
import AutoLink from '../base/AutoLink.vue'

const formatDate = useFormatDateTime({ dateStyle: 'long' })

export interface Article {
	path: string
	thumbnail: string
	title: string
	summary: string
	date: string
}

defineProps<{
	article: Article
}>()
</script>

<template>
	<AutoLink
		:to="article.path"
		class="active:scale-[0.99]! group transition-all ease-in-out cursor-pointer"
	>
		<article class="grid grid-cols-[1fr,2fr]">
			<img
				:src="article.thumbnail"
				class="aspect-video w-[100px] rounded-[4px] border-[1px] border-solid border-button-border object-cover"
			/>
			<div class="flex grow flex-col gap-0">
				<h3 class="m-0 text-base font-normal text-link leading-tight group-hover:underline">
					{{ article.title }}
				</h3>
				<div class="mt-auto text-sm">
					{{ formatDate(article.date) }}
				</div>
			</div>
		</article>
	</AutoLink>
</template>
