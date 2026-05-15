<script setup>
import { ref, onMounted } from 'vue'
import { get_taxphobia_news } from '@/helpers/taxphobia_news.js'

const news = ref([])
const isLoading = ref(false)
const error = ref(null)

onMounted(async () => {
	isLoading.value = true
	try {
		news.value = await get_taxphobia_news()
	} catch (err) {
		error.value = err
	} finally {
		isLoading.value = false
	}
})
</script>

<template>
	<div v-if="isLoading" class="text-center text-secondary text-sm p-4">
		Loading News...
	</div>
	<div v-else-if="error" class="text-center text-red-500 text-sm p-4">
		Failed to load news
	</div>
	<div v-else-if="news && news.length > 0" class="space-y-3 flex flex-col items-center w-full">
		<a
			v-for="(item, index) in news.slice(0, 4)"
			:key="`news-${index}`"
			:href="item.url || item.link || '#'"
			target="_blank"
			class="block w-full p-3 bg-button-bg hover:brightness-90 rounded-lg no-underline transition-all"
		>
			<h4 class="text-sm font-medium m-0 mb-1 text-contrast">{{ item.title }}</h4>
			<p v-if="item.excerpt || item.description || item.summary" class="text-xs text-secondary m-0 line-clamp-2">
				{{ item.excerpt || item.description || item.summary }}
			</p>
		</a>
		<a
			href="https://taxphobia.top/events"
			target="_blank"
			class="text-brand hover:underline text-sm"
		>
			View all news →
		</a>
	</div>
	<div v-else class="text-center text-secondary text-sm p-4">
		No news found
	</div>
</template>
