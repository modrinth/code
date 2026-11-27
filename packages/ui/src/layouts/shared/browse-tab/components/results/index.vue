<script setup lang="ts">
import LoadingIndicator from '#ui/components/base/LoadingIndicator.vue'
import Pagination from '#ui/components/base/Pagination.vue'
import ProjectCardList from '#ui/components/project/ProjectCardList.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { useResultActions } from '../../composables/use-result-actions'
import { injectBrowseManager } from '../../providers/browse-manager'
import ProjectResult from './project.vue'
import ServerResult from './server.vue'

defineProps<{
	dense?: boolean
}>()

const ctx = injectBrowseManager()
const { view } = ctx
const { formatMessage } = useVIntl()
const actionsFor = useResultActions(ctx)

const messages = defineMessages({
	offline: {
		id: 'browse.offline',
		defaultMessage: 'You are currently offline. Connect to the internet to browse Modrinth!',
	},
	noResults: {
		id: 'browse.no-results',
		defaultMessage: 'No results found for your query!',
	},
})
</script>

<template>
	<div class="search mt-1 [overflow-anchor:none]">
		<section v-if="view.loading.value" class="offline">
			<component :is="ctx.loadingComponent ?? LoadingIndicator" />
		</section>
		<section v-else-if="ctx.offline?.value && view.totalHits.value === 0" class="offline">
			{{ formatMessage(messages.offline) }}
		</section>
		<section v-else-if="view.hits.value.length === 0" class="offline text-center">
			<p>{{ formatMessage(messages.noResults) }}</p>
		</section>

		<ProjectCardList v-else :layout="view.layout.value">
			<template v-if="view.isServerType.value">
				<ServerResult
					v-for="hit in view.hits.value"
					:key="`server-card-${hit.project_id}`"
					:hit="hit"
					:actions="actionsFor(hit)"
					:dense="dense"
				/>
			</template>
			<template v-else>
				<ProjectResult
					v-for="hit in view.hits.value"
					:key="hit.project_id"
					:hit="hit"
					:actions="actionsFor(hit)"
				/>
			</template>
		</ProjectCardList>

		<div class="mt-3 flex justify-end">
			<Pagination
				:page="view.page.value"
				:count="view.pageCount.value"
				@switch-page="view.setPage"
			/>
		</div>
	</div>
</template>
