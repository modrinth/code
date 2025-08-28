<script setup lang="ts">
import { GitGraphIcon, RssIcon } from '@modrinth/assets'
import { articles as rawArticles } from '@modrinth/blog'
import { Avatar, ButtonStyled } from '@modrinth/ui'
import type { User } from '@modrinth/utils'
import dayjs from 'dayjs'
import { computed } from 'vue'

import NewsletterButton from '~/components/ui/NewsletterButton.vue'
import ShareArticleButtons from '~/components/ui/ShareArticleButtons.vue'

const config = useRuntimeConfig()
const route = useRoute()

const rawArticle = rawArticles.find((article) => article.slug === route.params.slug)

if (!rawArticle) {
	throw createError({
		fatal: true,
		statusCode: 404,
		message: 'The requested article could not be found.',
	})
}

const authorsUrl = `users?ids=${JSON.stringify(rawArticle.authors)}`

const [authors, html] = await Promise.all([
	rawArticle.authors
		? useAsyncData(authorsUrl, () => useBaseFetch(authorsUrl)).then((data) => {
				const users = data.data as Ref<User[]>
				users.value.sort((a, b) => {
					return rawArticle.authors.indexOf(a.id) - rawArticle.authors.indexOf(b.id)
				})

				return users
			})
		: Promise.resolve(),
	rawArticle.html(),
])

const article = computed(() => ({
	...rawArticle,
	path: `/news/${rawArticle.slug}`,
	thumbnail: rawArticle.thumbnail
		? `/news/article/${rawArticle.slug}/thumbnail.webp`
		: `/news/default.webp`,
	title: rawArticle.title,
	summary: rawArticle.summary,
	date: rawArticle.date,
	html,
}))

const authorCount = computed(() => authors?.value?.length ?? 0)

const articleTitle = computed(() => article.value.title)
const articleUrl = computed(() => `https://modrinth.com/news/article/${route.params.slug}`)

const thumbnailPath = computed(() =>
	article.value.thumbnail
		? `${config.public.siteUrl}${article.value.thumbnail}`
		: `${config.public.siteUrl}/news/default.jpg`,
)

const dayjsDate = computed(() => dayjs(article.value.date))

useSeoMeta({
	title: () => `${articleTitle.value} - Modrinth News`,
	ogTitle: () => articleTitle.value,
	description: () => article.value.summary,
	ogDescription: () => article.value.summary,
	ogType: 'article',
	ogImage: () => thumbnailPath.value,
	articlePublishedTime: () => dayjsDate.value.toISOString(),
	twitterCard: 'summary_large_image',
	twitterImage: () => thumbnailPath.value,
})
</script>

<template>
	<div class="page experimental-styles-within py-6">
		<div
			class="flex flex-wrap items-center justify-between gap-4 border-0 border-b-[1px] border-solid border-divider px-6 pb-6"
		>
			<nuxt-link :to="`/news`">
				<h1 class="m-0 text-3xl font-extrabold hover:underline">News</h1>
			</nuxt-link>
			<div class="flex gap-2">
				<NewsletterButton />
				<ButtonStyled circular>
					<a v-tooltip="`RSS feed`" aria-label="RSS feed" href="/news/feed/rss.xml" target="_blank">
						<RssIcon />
					</a>
				</ButtonStyled>
				<ButtonStyled circular icon-only>
					<a v-tooltip="`Changelog`" href="/news/changelog" aria-label="Changelog">
						<GitGraphIcon />
					</a>
				</ButtonStyled>
			</div>
		</div>
		<article class="mt-6 flex flex-col gap-4 px-6">
			<h2 class="m-0 text-2xl font-extrabold leading-tight sm:text-4xl">
				{{ article.title }}
			</h2>
			<p class="m-0 text-base leading-tight sm:text-lg">{{ article.summary }}</p>
			<div class="mt-auto flex flex-wrap items-center gap-1 text-sm text-secondary sm:text-base">
				<template v-for="(author, index) in authors" :key="`author-${author.id}`">
					<span v-if="authorCount - 1 === index && authorCount > 1">and</span>
					<span class="flex items-center">
						<nuxt-link
							:to="`/user/${author.id}`"
							class="inline-flex items-center gap-1 font-semibold hover:underline hover:brightness-[--hover-brightness]"
						>
							<Avatar :src="author.avatar_url" circle size="24px" />
							{{ author.username }}
						</nuxt-link>
						<span v-if="(authors?.length ?? 0) > 2 && index !== authorCount - 1">,</span>
					</span>
				</template>
				<template v-if="!authors || authorCount === 0">
					<nuxt-link
						to="/organization/modrinth"
						class="inline-flex items-center gap-1 font-semibold hover:underline hover:brightness-[--hover-brightness]"
					>
						<Avatar src="https://cdn-raw.modrinth.com/modrinth-icon-96.webp" size="24px" />
						Modrinth Team
					</nuxt-link>
				</template>
				<span class="hidden md:block">•</span>
				<span class="hidden md:block"> {{ dayjsDate.format('MMMM D, YYYY') }}</span>
			</div>
			<span class="text-sm text-secondary sm:text-base md:hidden">
				Posted on {{ dayjsDate.format('MMMM D, YYYY') }}</span
			>
			<ShareArticleButtons :title="article.title" :url="articleUrl" />
			<img
				:src="article.thumbnail"
				class="aspect-video w-full rounded-xl border-[1px] border-solid border-button-border object-cover sm:rounded-2xl"
				:alt="article.title"
			/>
			<div class="markdown-body" v-html="article.html" />
			<h3
				class="mb-0 mt-4 border-0 border-t-[1px] border-solid border-divider pt-4 text-base font-extrabold sm:text-lg"
			>
				Share this article
			</h3>
			<ShareArticleButtons :title="article.title" :url="articleUrl" />
		</article>
	</div>
</template>

<style lang="scss" scoped>
.page {
	> *:not(.full-width-bg),
	> .full-width-bg > * {
		max-width: 56rem;
		margin-inline: auto;
	}
}

.brand-gradient-bg {
	background: var(--brand-gradient-bg);
	border-color: var(--brand-gradient-border);
}

@media (max-width: 640px) {
	.page {
		padding-top: 1rem;
		padding-bottom: 1rem;
	}

	article {
		gap: 1rem;
	}
}

:deep(.markdown-body) {
	h1,
	h2 {
		border-bottom: none;
		padding: 0;
	}

	ul,
	ol > li:not(:last-child) {
		margin-bottom: 0.5rem;
	}

	ul,
	ol {
		p {
			margin-bottom: 0.5rem;
		}
	}

	ul,
	ol {
		strong {
			color: var(--color-contrast);
			font-weight: 600;
		}
	}

	h1,
	h2,
	h3 {
		margin-bottom: 0.5rem;
	}

	h1 {
		font-size: 1.5rem;
		@media (min-width: 640px) {
			font-size: 2rem;
		}
	}

	h2 {
		font-size: 1.25rem;
		@media (min-width: 640px) {
			font-size: 1.5rem;
		}
	}

	h3 {
		font-size: 1.125rem;
		@media (min-width: 640px) {
			font-size: 1.25rem;
		}
	}

	p {
		margin-bottom: 1.25rem;
		font-size: 0.875rem;
		@media (min-width: 640px) {
			font-size: 1rem;
		}
	}

	a {
		color: var(--color-brand);
		font-weight: 600;

		&:hover {
			text-decoration: underline;
		}
	}

	h1,
	h2 {
		a {
			font-weight: 800;
		}
	}

	h1,
	h2,
	h3,
	h4,
	h5,
	h6 {
		a {
			color: var(--color-contrast);
		}
	}

	img {
		border: 1px solid var(--color-button-border);
		border-radius: var(--radius-md);
		@media (min-width: 640px) {
			border-radius: var(--radius-lg);
		}
	}

	> img,
	> :has(img:first-child:last-child) {
		display: flex;
		justify-content: center;
	}

	@media (max-width: 640px) {
		h1,
		h2,
		h3,
		h4,
		h5,
		h6 {
			margin-bottom: 0.5rem;
		}

		p {
			margin-bottom: 1rem;
		}

		ul,
		ol {
			padding-left: 1.25rem;
		}

		pre {
			overflow-x: auto;
			font-size: 0.75rem;
		}

		table {
			display: block;
			overflow-x: auto;
			white-space: nowrap;
		}
	}
}
</style>
