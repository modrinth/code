<template>
	<div class="mx-2 p-4 !py-8 sm:mx-8 sm:p-32">
		<div class="my-8 flex items-center justify-between">
			<h2 class="m-0 mx-auto text-3xl font-extrabold sm:text-4xl">
				{{ formatMessage(messages.latestNews) }}
			</h2>
		</div>

		<div v-if="latestArticles" class="grid grid-cols-[repeat(auto-fit,minmax(250px,1fr))] gap-4">
			<div
				v-for="(article, index) in latestArticles"
				:key="article.slug"
				:class="{ 'max-xl:hidden': index === 2 }"
			>
				<NewsArticleCard :article="article" />
			</div>
		</div>
		<div class="mx-2 my-8 flex w-full items-center justify-center">
			<ButtonStyled color="brand" size="large">
				<nuxt-link to="/news">
					<NewspaperIcon />
					{{ formatMessage(messages.viewAll) }}
				</nuxt-link>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import { NewspaperIcon } from '@modrinth/assets'
import { articles as rawArticles } from '@modrinth/blog'
import { ButtonStyled, NewsArticleCard } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed, ref } from 'vue'

const { formatMessage } = useVIntl()

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
		}))
		.sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime()),
)

const messages = defineMessages({
	latestNews: {
		id: 'ui.latest-news-row.latest-news',
		defaultMessage: 'Latest news from Modrinth',
	},
	viewAll: {
		id: 'ui.latest-news-row.view-all',
		defaultMessage: 'View all news',
	},
})

const latestArticles = computed(() => articles.value.slice(0, 3))
</script>
