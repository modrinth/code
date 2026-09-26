<template>
	<div class="review-details -mt-1 min-w-0 pb-10 text-sm text-secondary">
		<p v-if="isLoading" role="status" class="m-0">
			{{ formatMessage(messages.loading) }}
		</p>
		<div v-else-if="error" role="alert">
			<p class="m-0">{{ formatMessage(messages.loadError) }}</p>
			<Button @click="refresh">{{ formatMessage(messages.retry) }}</Button>
		</div>
		<p v-else-if="!project" class="m-0">{{ formatMessage(messages.empty) }}</p>
		<template v-else>
			<Title />
			<Slug />
			<Icon />
			<Summary />
			<License />
			<Tags />
			<Links />
			<Compatibility v-if="!project.minecraft_server" />
			<ServerDetails v-else />
			<Members />
			<Details />
		</template>
	</div>
</template>

<script setup lang="ts">
import { Button, useVIntl } from '@modrinth/ui'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../messages'
import Members from './members/index.vue'
import Compatibility from './project/compatibility.vue'
import Details from './project/details.vue'
import Icon from './project/icon.vue'
import License from './project/license.vue'
import Links from './project/links.vue'
import ServerDetails from './project/server-details.vue'
import Slug from './project/slug.vue'
import Summary from './project/summary.vue'
import Tags from './project/tags.vue'
import Title from './project/title.vue'

const { project, isLoading, error, refresh } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
</script>

<style scoped>
.review-details > :deep(.review-section:first-child) {
	@apply border-t-0;
}
.review-details :deep(.review-badge) {
	@apply rounded bg-button-bg px-1.5 py-0.5 text-[0.6875rem];
}
</style>
