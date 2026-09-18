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
				:label="formatMessage(messages.projectTitle)"
				class="review-section"
			>
				<h3 class="review-section-label">
					{{ formatMessage(messages.projectTitle) }}
				</h3>
				<h1 class="m-0 text-xl">{{ project.name }}</h1>
			</ReviewPanel>
			<ReviewPanel
				mode="anchored"
				as="section"
				:target="{ kind: 'slug' }"
				:label="formatMessage(messages.slug)"
				class="review-section"
			>
				<h3 class="review-section-label">
					{{ formatMessage(messages.slug) }}
				</h3>
				<Slug />
			</ReviewPanel>
			<ReviewPanel
				mode="anchored"
				as="section"
				:target="{ kind: 'icon' }"
				:label="formatMessage(messages.icon)"
				class="review-section review-section-icon"
			>
				<h3 class="review-section-label">
					{{ formatMessage(messages.icon) }}
				</h3>
				<Icon />
			</ReviewPanel>
			<ReviewPanel
				mode="anchored"
				as="section"
				:target="{ kind: 'summary' }"
				:label="formatMessage(messages.summary)"
				class="review-section"
			>
				<h3 class="review-section-label">
					{{ formatMessage(messages.summary) }}
				</h3>
				<Summary />
			</ReviewPanel>
			<section class="review-section" :aria-label="formatMessage(messages.license)">
				<h3 class="review-section-label">
					{{ formatMessage(messages.license) }}
				</h3>
				<License />
			</section>
			<section class="review-section" :aria-label="formatMessage(messages.tags)">
				<h3 class="review-section-label">
					{{ formatMessage(messages.tags) }}
				</h3>
				<Tags v-if="hasTags" />
				<span v-else>{{ formatMessage(messages.emptyTags) }}</span>
			</section>
			<section :aria-label="formatMessage(messages.links)" class="review-section">
				<h3 class="review-section-label">
					{{ formatMessage(messages.links) }}
				</h3>
				<Links v-if="hasLinks" />
				<span v-else>{{ formatMessage(messages.emptyLinks) }}</span>
			</section>
			<section class="review-section">
				<h3 class="review-section-label">
					{{ formatMessage(messages.compatibility) }}
				</h3>
				<Compatibility />
			</section>
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
import Members from './project/members.vue'
import Slug from './project/slug.vue'
import Summary from './project/summary.vue'
import Tags from './project/tags.vue'

const { project, isLoading, error, refresh } = injectProjectReviewPageContext()
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
	@apply flex flex-col gap-3 border-0 border-t border-solid border-divider px-0.5 pb-4 pt-3;
}
.review-section:first-child {
	@apply border-t-0;
}
.review-section-icon {
	@apply items-start gap-3;
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
