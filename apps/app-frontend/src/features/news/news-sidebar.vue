<script setup lang="ts">
import { NewspaperIcon } from '@modrinth/assets'
import { ButtonLink, defineMessages, NewsArticleCard, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { watch } from 'vue'

import { newsArticlesQueryOptions } from './queries'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	news: {
		id: 'app.news.title',
		defaultMessage: 'News',
	},
	viewAllNews: {
		id: 'app.news.view-all',
		defaultMessage: 'View all news',
	},
})

const { data: news, error } = useQuery({
	...newsArticlesQueryOptions(),
	select: (articles) => articles.slice(0, 4),
})

watch(error, (error) => {
	if (error) console.error('Failed to fetch news articles', error)
})
</script>

<template>
	<div v-if="news && news.length > 0" class="p-4 flex flex-col items-center">
		<h3 class="text-base mb-4 text-primary font-medium m-0 text-left w-full">
			{{ formatMessage(messages.news) }}
		</h3>
		<div class="space-y-4 flex flex-col items-center w-full">
			<NewsArticleCard v-for="item in news" :key="item.path" :article="item" />
			<ButtonLink
				type="colored"
				color="brand"
				size="xl"
				href="https://modrinth.com/news"
				target="_blank"
				class="my-4"
			>
				<NewspaperIcon />
				{{ formatMessage(messages.viewAllNews) }}
			</ButtonLink>
		</div>
	</div>
</template>
