<template>
	<section>
		<h2>{{ formatMessage(messages.compatibility) }}</h2>
		<dl class="m-0 grid grid-cols-[5rem_minmax(0,1fr)] gap-x-3 gap-y-2">
			<dt>{{ formatMessage(messages.gameVersions) }}</dt>
			<dd class="m-0 flex flex-wrap items-start gap-1">
				<span v-for="version in projectV2?.game_versions" :key="version" class="review-badge">{{
					version
				}}</span>
				<span v-if="compatibilityError">{{ formatMessage(messages.unavailable) }}</span>
			</dd>
			<dt>{{ formatMessage(messages.platforms) }}</dt>
			<dd class="m-0 flex flex-wrap items-start gap-1">
				<span v-for="loader in project?.loaders" :key="loader" class="review-badge">{{
					formatLoader(formatMessage, loader)
				}}</span>
			</dd>
			<dt>{{ formatMessage(messages.environments) }}</dt>
			<dd class="m-0 flex flex-wrap items-start gap-1">
				<span v-for="environment in project?.environment" :key="environment" class="review-badge">{{
					formatMessage(ENVIRONMENTS_COPY[environment].title)
				}}</span>
			</dd>
		</dl>
	</section>
</template>

<script setup lang="ts">
import { ENVIRONMENTS_COPY, formatLoader, useVIntl } from '@modrinth/ui'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../../messages'

const { project, projectV2, compatibilityError } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
</script>
