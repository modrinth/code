<template>
	<div class="flex flex-col gap-6 rounded-2xl border border-solid border-surface-5 p-4">
		<div class="flex flex-col gap-2">
			<label class="font-semibold text-contrast">Project</label>
			<ProjectCombobox
				v-model="ctx.selectedProjectId.value"
				:project-types="['modpack']"
				placeholder="Select modpack"
				search-placeholder="Search by name or paste ID..."
				loading-message="Loading..."
				no-results-message="No results found"
			/>
		</div>

		<div v-if="ctx.selectedProjectId.value" class="flex flex-col gap-2">
			<label class="font-semibold text-contrast">Version</label>
			<Combobox
				v-model="ctx.selectedVersionId.value"
				placeholder="Select version"
				:options="versionOptions"
				:searchable="true"
				search-placeholder="Search versions..."
				:no-options-message="versionsLoading ? 'Loading...' : 'No results found'"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	Combobox,
	injectModrinthClient,
	injectNotificationManager,
	ProjectCombobox,
} from '@modrinth/ui'
import { computed, ref, watch } from 'vue'

import { injectServerCompatibilityContext } from '../server-compatibility-modal'

const ctx = injectServerCompatibilityContext()
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

watch(
	() => ctx.selectedProjectId.value,
	async (newProjectId) => {
		ctx.selectedVersionId.value = ''
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
