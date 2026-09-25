<template>
	<div
		ref="hotkeyScope"
		class="group/versions flex h-full min-h-0 flex-col gap-2.5 overflow-hidden"
	>
		<CreateProjectVersionModal
			v-if="editorHost"
			:key="selection"
			ref="editModal"
			:host="editorHost"
			:enable-drop-area="false"
		/>
		<ReviewPanel
			v-if="resolve({ kind: 'undefined-project' })"
			mode="inline"
			:target="{ kind: 'undefined-project' }"
		/>
		<ReviewPanel
			mode="inline"
			:target="{ kind: 'versions' }"
			:hotkey-scope="hotkeyScope"
			:disabled="
				isLoading || !!error || versionsQuery.isPending.value || versionsQuery.isError.value
			"
			class="group-hover/versions:opacity-100"
		>
		</ReviewPanel>
		<div class="min-h-0 flex-1 overflow-auto">
			<div class="mr-0.5 mt-0.5 flex flex-col gap-1">
				<div class="flex justify-between">
					<div class="flex flex-wrap items-center gap-x-6 gap-y-2 pl-2.5">
						<h2 class="m-0 flex items-center gap-2 text-sm font-semibold text-secondary">
							{{ formatMessage(messages.versions) }}
							<span class="text-contrast">{{ versions.length }}</span>
						</h2>
						<span class="flex items-center gap-2 text-sm font-semibold text-secondary">
							{{ formatMessage(messages.files) }}
							<span class="text-contrast">{{ fileCount }}</span>
						</span>
						<span class="flex items-center gap-2 text-sm font-semibold text-secondary">
							{{ formatMessage(messages.withheld) }}
							<span :class="withheldCount ? 'text-orange' : 'text-contrast'">{{
								withheldCount
							}}</span>
						</span>
					</div>
					<Button
						v-if="versions.length && !isLoading && !error && !versionsQuery.isError.value"
						type="quiet"
						@click="
							expanded = allExpanded ? new Set() : new Set(versions.map((version) => version.id))
						"
					>
						<ListChevronsDownUpIcon v-if="allExpanded" aria-hidden="true" />
						<ListChevronsUpDownIcon v-else aria-hidden="true" />
						{{ formatMessage(allExpanded ? messages.collapseAll : messages.expandAll) }}
					</Button>
				</div>
				<p v-if="!selection" class="m-0 text-secondary">
					{{ formatMessage(messages.empty) }}
				</p>
				<div v-else-if="error || versionsQuery.isError.value" role="alert">
					<p class="m-0">{{ formatMessage(messages.loadError) }}</p>
					<Button @click="retry">{{ formatMessage(messages.retry) }}</Button>
				</div>
				<p v-else-if="isLoading || versionsQuery.isPending.value" role="status" class="m-0">
					{{ formatMessage(messages.loading) }}
				</p>
				<p v-else-if="!versions.length" class="m-0 text-secondary">
					{{ formatMessage(messages.emptyVersions) }}
				</p>
				<template v-else>
					<VersionCard
						v-for="version in versions"
						:key="version.id"
						:version="version"
						:expanded="expandedIds.has(version.id)"
						:editable="!!editorHost"
						@edit="editModal?.openEditVersionModal(version.id, version.project_id, $event)"
						@toggle="toggle(version.id)"
					/>
				</template>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ListChevronsDownUpIcon, ListChevronsUpDownIcon } from '@modrinth/assets'
import { Button, useVIntl } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, ref, useTemplateRef } from 'vue'

import CreateProjectVersionModal from '~/components/ui/create-project-version/CreateProjectVersionModal.vue'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewPanels } from '~/providers/project-review/review-panels'
import type { ManageVersionHost } from '~/providers/version/manage-version-modal'

import { projectReviewMessages as messages } from '../messages'
import ReviewPanel from '../review-panel/index.vue'
import VersionCard from './version-card.vue'

const { formatMessage } = useVIntl()
const { resolve } = injectReviewPanels()
const hotkeyScope = useTemplateRef<HTMLElement>('hotkeyScope')
const { selection, projectV2, versions, versionsQuery, isLoading, error, refresh } =
	injectProjectReviewPageContext()
const queryClient = useQueryClient()
const editModal = useTemplateRef<InstanceType<typeof CreateProjectVersionModal>>('editModal')
const editorHost = computed<ManageVersionHost | undefined>(() => {
	const project = projectV2.value
	if (!project || isLoading.value || error.value) return undefined
	return {
		projectV2: computed(() => project),
		invalidate: async (projectId, versionId) => {
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: ['project', projectId] }),
				queryClient.invalidateQueries({
					queryKey: ['project', 'v2', projectId],
				}),
				queryClient.invalidateQueries({
					queryKey: ['project', 'v3', projectId],
				}),
				queryClient.invalidateQueries({
					queryKey: ['version', 'v3', versionId],
				}),
				queryClient.invalidateQueries({
					queryKey: ['project-attribution', projectId],
				}),
				queryClient.invalidateQueries({
					queryKey: ['tech-review-project-report', projectId],
				}),
				queryClient.invalidateQueries({ queryKey: ['tech-reviews'] }),
			])
		},
	}
})
const expanded = ref<Set<string> | null>(null)
const expandedIds = computed(
	() => expanded.value ?? new Set(versions.value[0] ? [versions.value[0].id] : []),
)
const fileCount = computed(() =>
	versions.value.reduce((count, version) => count + version.files.length, 0),
)
const withheldCount = computed(() =>
	versions.value.reduce(
		(count, version) => count + (version.files_missing_attribution?.length ?? 0),
		0,
	),
)
const allExpanded = computed(() =>
	versions.value.every((version) => expandedIds.value.has(version.id)),
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
