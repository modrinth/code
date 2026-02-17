<template>
	<div class="5 flex flex-col gap-2">
		<div class="font-semibold text-contrast">Select modpack</div>

		<div class="flex flex-col gap-6 rounded-2xl border border-solid border-surface-5 p-4">
			<div class="flex flex-col gap-2">
				<label class="font-semibold text-contrast">Project</label>
				<ProjectCombobox
					v-model="selectedProjectId"
					:project-types="['modpack']"
					placeholder="Select modpack"
					search-placeholder="Search by name or paste ID..."
					loading-message="Loading..."
					no-results-message="No results found"
				/>
			</div>

			<div v-if="selectedProjectId" class="flex flex-col gap-2">
				<label class="font-semibold text-contrast">Version</label>
				<Combobox
					v-model="selectedVersionId"
					placeholder="Select version"
					:options="versionOptions"
					:searchable="true"
					search-placeholder="Search versions..."
					:no-options-message="versionsLoading ? 'Loading...' : 'No results found'"
				/>
			</div>
		</div>
		<div v-if="selectedVersion" class="flex flex-col gap-4 rounded-2xl bg-surface-2 p-4">
			<div class="flex items-center justify-between">
				<div class="text-secondary">Game version</div>
				<div class="flex flex-wrap gap-1">
					<TagItem v-for="gv in selectedVersion.game_versions" :key="gv">
						{{ gv }}
					</TagItem>
				</div>
			</div>

			<div class="flex items-center justify-between">
				<div class="text-secondary">Platform</div>
				<div class="flex flex-wrap gap-1">
					<TagItem
						v-for="loader in selectedVersion.loaders"
						:key="loader"
						:style="`--_color: var(--color-platform-${loader})`"
					>
						<component :is="getLoaderIcon(loader)" v-if="getLoaderIcon(loader)" class="h-4 w-4" />
						<FormattedTag :tag="loader" enforce-type="loader" />
					</TagItem>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { getLoaderIcon } from '@modrinth/assets'
import {
	Combobox,
	FormattedTag,
	injectModrinthClient,
	injectNotificationManager,
	ProjectCombobox,
	TagItem,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import { injectServerCompatibilityContext } from '../server-compatibility-modal'

const { selectedProjectId, selectedVersionId } = injectServerCompatibilityContext()
const { labrinth } = injectModrinthClient()
const { addNotification } = injectNotificationManager()

interface VersionInfo {
	id: string
	version_number: string
	name: string
}

const versionsLoading = ref(false)
const projectVersions = ref<VersionInfo[]>([])

const versionOptions = computed(() =>
	projectVersions.value.map((v) => ({
		label: v.version_number,
		value: v.id,
	})),
)

const { data: selectedVersion } = useQuery({
	queryKey: computed(() => ['versions', 'detail', selectedVersionId.value]),
	queryFn: () => labrinth.versions_v3.getVersion(selectedVersionId.value),
	enabled: computed(() => !!selectedVersionId.value),
})

watch(
	() => selectedProjectId.value,
	async (newProjectId) => {
		selectedVersionId.value = ''
		projectVersions.value = []

		if (!newProjectId) return

		versionsLoading.value = true
		try {
			const versions = await labrinth.versions_v3.getProjectVersions(newProjectId)
			projectVersions.value = versions.map((v) => ({
				id: v.id,
				version_number: v.version_number,
				name: v.name,
			}))
		} catch (error: unknown) {
			const err = error as { data?: { description?: string } }
			addNotification({
				title: 'Failed to load versions',
				text: err.data?.description || String(error),
				type: 'error',
			})
		} finally {
			versionsLoading.value = false
		}
	},
)
</script>
