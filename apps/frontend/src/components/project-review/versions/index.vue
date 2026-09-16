<template>
	<div class="flex flex-col gap-3">
		<p v-if="!selection" class="text-secondary">
			{{ formatMessage(messages.empty) }}
		</p>
		<div v-else-if="error || versionsQuery.isError.value" role="alert">
			<p>{{ formatMessage(messages.loadError) }}</p>
			<Button @click="retry">{{ formatMessage(messages.retry) }}</Button>
		</div>
		<p v-else-if="isLoading || versionsQuery.isPending.value" role="status">
			{{ formatMessage(messages.loading) }}
		</p>
		<p v-else-if="!versions.length" class="text-secondary">
			{{ formatMessage(messages.emptyVersions) }}
		</p>
		<template v-else>
			<div class="flex flex-wrap justify-end gap-2">
				<Button @click="expanded = new Set(versions.map((version) => version.id))">{{
					formatMessage(messages.expandAll)
				}}</Button>
				<Button @click="expanded = new Set()">{{ formatMessage(messages.collapseAll) }}</Button>
			</div>
			<VersionCard
				v-for="version in versions"
				:key="version.id"
				:version="version"
				:expanded="expandedIds.has(version.id)"
				@toggle="toggle(version.id)"
			/>
		</template>
	</div>
</template>

<script setup lang="ts">
import { Button, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../messages'
import VersionCard from './version-card.vue'

const { formatMessage } = useVIntl()
const { selection, versions, versionsQuery, isLoading, error, refresh } =
	injectProjectReviewPageContext()
const expanded = ref<Set<string> | null>(null)
const expandedIds = computed(
	() => expanded.value ?? new Set(versions.value[0] ? [versions.value[0].id] : []),
)
function toggle(id: string) {
	const next = new Set(expandedIds.value)
	if (next.has(id)) next.delete(id)
	else next.add(id)
	expanded.value = next
}
async function retry() {
	await Promise.all([refresh(), versionsQuery.refetch()])
}
</script>
