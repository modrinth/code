<template>
	<div class="mx-auto max-w-[1280px] border border-solid border-[#c3c3c3] bg-[#EFEFEF] p-4">
		<div>
			<h2 class="m-0 mb-2 text-lg font-bold">News feed</h2>
		</div>

		<div v-if="latestArticles" class="flex flex-col gap-2">
			<div v-for="article in latestArticles" :key="article.slug">
				<NewsArticleListItem :article="article" />
			</div>
		</div>
		<div class="mt-4">
			<nuxt-link to="/news" class="text-link"> Read more articles ►</nuxt-link>
		</div>
	</div>
</template>

<script setup lang="ts">
import { articles as rawArticles } from '@modrinth/blog'
import { NewsArticleListItem } from '@modrinth/ui'
import { computed, ref } from 'vue'

const articles = ref(
	rawArticles
		.map((article) => ({
			...article,
			path: `/news/article/${article.slug}/`,
			thumbnail: article.thumbnail
				? `/news/article/${article.slug}/thumbnail.webp`
				: `/news/default.webp`,
			title: article.title,
			summary: article.summary,
			date: article.date,
			unlisted: article.unlisted,
		}))
		.filter((a) => !a.unlisted)
		.sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime()),
)

const latestArticles = computed(() => articles.value.slice(0, 10))
</script>
