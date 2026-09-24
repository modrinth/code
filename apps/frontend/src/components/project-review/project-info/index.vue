<template>
	<div class="review-details min-w-0 pb-10 text-sm text-secondary">
		<p v-if="isLoading" role="status" class="m-0">
			{{ formatMessage(messages.loading) }}
		</p>
		<div v-else-if="error" role="alert">
			<p class="m-0">{{ formatMessage(messages.loadError) }}</p>
			<Button @click="refresh">{{ formatMessage(messages.retry) }}</Button>
		</div>
		<p v-else-if="!project" class="m-0">{{ formatMessage(messages.empty) }}</p>
		<template v-else>
			<ReviewPanel
				mode="anchored"
				as="section"
				:target="{ kind: 'title' }"
				class="review-section !pt-1"
			>
				<h3 class="review-section-label">
					{{ formatMessage(messages.titleAndSlug) }}
				</h3>
				<h1 class="m-0 text-xl">
					<NuxtLink
						:to="`/${projectV2?.project_type ?? project.project_types[0]}/${project.slug ?? project.id}`"
						target="_blank"
						rel="noopener noreferrer"
						class="hover:underline"
					>
						{{ project.name }}
					</NuxtLink>
				</h1>
				<Slug />
			</ReviewPanel>
			<ReviewPanel mode="anchored" as="section" :target="{ kind: 'icon' }" class="review-section">
				<h3 class="review-section-label">
					{{ formatMessage(messages.icon) }}
				</h3>
				<Icon />
			</ReviewPanel>
			<ReviewPanel
				mode="anchored"
				as="section"
				:target="{ kind: 'summary' }"
				class="review-section"
			>
				<h3 class="review-section-label">
					{{ formatMessage(messages.summary) }}
				</h3>
				<Summary />
			</ReviewPanel>
			<ReviewPanel
				mode="anchored"
				as="section"
				:target="{ kind: 'license' }"
				class="review-section"
				:aria-label="formatMessage(messages.license)"
			>
				<h3 class="review-section-label">
					{{ formatMessage(messages.license) }}
				</h3>
				<License />
			</ReviewPanel>
			<ReviewPanel mode="anchored" as="section" :target="{ kind: 'tags' }" class="review-section">
				<h3 class="review-section-label">
					{{ formatMessage(messages.tags) }}
				</h3>
				<Tags v-if="hasTags" />
				<span v-else>{{ formatMessage(messages.emptyTags) }}</span>
			</ReviewPanel>
			<section :aria-label="formatMessage(messages.links)" class="review-section">
				<h3 class="review-section-label">
					{{ formatMessage(messages.links) }}
				</h3>
				<Links v-if="hasLinks" />
				<span v-else>{{ formatMessage(messages.emptyLinks) }}</span>
			</section>
			<ReviewPanel
				mode="anchored"
				as="section"
				:target="{ kind: 'compatibility' }"
				class="review-section"
			>
				<h3 class="review-section-label">
					{{ formatMessage(messages.compatibility) }}
				</h3>
				<Compatibility />
			</ReviewPanel>
			<section class="review-section">
				<h3 class="review-section-label">
					{{ formatMessage(messages.members) }}
				</h3>
				<Members />
			</section>
			<section class="review-section">
				<h3 class="review-section-label">
					{{ formatMessage(messages.details) }}
				</h3>
				<Details />
			</section>
		</template>
	</div>
</template>

<script setup lang="ts">
import { Button, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'
import { reviewExternalUrl } from '~/providers/project-review/project-links'

import { projectReviewMessages as messages } from '../messages'
import ReviewPanel from '../review-panel/index.vue'
import Compatibility from './project/compatibility.vue'
import Details from './project/details.vue'
import Icon from './project/icon.vue'
import License from './project/license.vue'
import Links from './project/links.vue'
import Members from './members/index.vue'
import Slug from './project/slug.vue'
import Summary from './project/summary.vue'
import Tags from './project/tags.vue'

const { project, projectV2, isLoading, error, refresh } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
const hasTags = computed(
	() => !!(project.value?.categories.length || project.value?.additional_categories.length),
)
const hasLinks = computed(() =>
	Object.values(project.value?.link_urls ?? {}).some((link) => reviewExternalUrl(link.url)),
)
</script>

<style scoped>
.review-section {
	@apply flex flex-col gap-2 border-0 border-t border-solid border-divider px-0.5 pb-3 pt-3;
}
.review-section:first-child {
	@apply border-t-0;
}
.review-section-label {
	@apply m-0 text-sm font-normal text-secondary;
}
.review-details :deep(.review-badge) {
	border-radius: 0.25rem;
	background: var(--color-button-bg);
	padding: 0.125rem 0.375rem;
	font-size: 0.6875rem;
}
</style>
