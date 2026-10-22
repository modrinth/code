<template>
	<section class="flex flex-col gap-3 rounded-lg bg-surface-2 p-3 text-xs" aria-live="polite">
		<div class="flex min-w-0 items-center gap-1 font-semibold text-contrast">
			<GlobeIcon class="size-4 shrink-0" aria-hidden="true" />
			<span>{{ match.platform }} · {{ linkInfoLabels[match.kind] }}</span>
		</div>
		<p
			v-for="message in validationMessages"
			:key="message.code"
			class="m-0 flex items-start gap-1 text-orange"
		>
			<TriangleAlertIcon class="size-4 shrink-0" aria-hidden="true" />
			{{ message.text }}
		</p>
		<p
			v-for="warning in data?.warnings"
			:key="warning"
			class="m-0 flex items-start gap-2 text-orange"
		>
			<TriangleAlertIcon class="size-4 shrink-0" aria-hidden="true" />
			{{ linkInfoLabels[warning] }}
		</p>
		<div v-for="(profile, index) in data?.profiles" :key="index" class="flex flex-col gap-2">
			<div class="flex items-center gap-2">
				<img
					v-if="safeImage(profile.avatar)"
					:src="safeImage(profile.avatar)"
					alt=""
					class="size-6 rounded object-cover"
					referrerpolicy="no-referrer"
				/>
				<a
					v-if="reviewExternalUrl(profile.url)"
					:href="reviewExternalUrl(profile.url)"
					target="_blank"
					rel="noopener noreferrer"
					class="min-w-0 break-words font-semibold text-contrast hover:underline"
				>
					{{ profile.title }}
				</a>
				<strong v-else class="min-w-0 break-words text-contrast">{{ profile.title }}</strong>
			</div>
			<span v-if="profile.label" class="text-secondary">{{ linkInfoLabels[profile.label] }}</span>
			<p v-if="profile.description" class="m-0 whitespace-pre-wrap break-words text-secondary">
				{{ profile.description }}
			</p>
			<div v-if="profile.details.length" class="flex flex-wrap items-center gap-1.5">
				<DetailBadge v-for="entry in profile.details" :key="entry.label" :entry="entry" />
			</div>
		</div>
		<p v-if="!data?.profiles.length && match.name" class="m-0 break-words">
			{{ match.name }}
		</p>
		<p v-if="isFetching" class="m-0 text-xs text-secondary">Loading link details…</p>
		<p v-else-if="error" class="m-0 text-xs text-secondary">
			{{
				notFound
					? 'This resource was not found. It may be private, deleted, or expired.'
					: 'Additional details could not be loaded. The service may be unavailable, private, rate limited, or blocked by the browser.'
			}}
		</p>
		<p v-else-if="!match.load" class="m-0 text-xs text-secondary">
			No additional public metadata is available for this link.
		</p>
		<details v-if="data?.links?.length" class="text-xs">
			<summary class="cursor-pointer">External links on this page</summary>
			<ul class="m-0 mt-2 flex list-none flex-col gap-2 p-0">
				<li v-for="link in data.links" :key="link.url">
					<a :href="link.url" target="_blank" rel="noopener noreferrer" class="break-all">{{
						link.title
					}}</a>
				</li>
			</ul>
		</details>
	</section>
</template>

<script setup lang="ts">
import { ModrinthApiError } from '@modrinth/api-client'
import { GlobeIcon, TriangleAlertIcon } from '@modrinth/assets'
import { injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { reviewExternalUrl } from '~/providers/project-review/project-links'

import DetailBadge from './detail-badge.vue'
import { linkInfoLabels } from './labels'
import { identifyLink, linkInfoQueryOptions } from './query'
import type { LinkValidationWarning } from './validation'

const props = defineProps<{
	url: string
	enabled: boolean
	validationMessages?: LinkValidationWarning[]
}>()
const client = injectModrinthClient()

const match = computed(() => identifyLink(props.url))
const { data, error, isFetching } = useQuery(
	computed(() => ({
		...linkInfoQueryOptions(props.url, match.value, client),
		enabled: import.meta.client && props.enabled && !!match.value.load,
	})),
)
const notFound = computed(
	() => error.value instanceof ModrinthApiError && error.value.statusCode === 404,
)

function safeImage(value: string | null | undefined) {
	return reviewExternalUrl(value ?? undefined)
}
</script>
