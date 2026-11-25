<template>
	<div class="flex flex-col gap-6">
		<DataLossWarningBanner />
		<div class="flex flex-col gap-2">
			<div class="font-semibold text-contrast">{{ formatMessage(messages.selectModpack) }}</div>

			<div class="flex flex-col gap-6 rounded-2xl border border-solid border-surface-5 p-4">
				<Field :label="formatMessage(messages.project)" :input-id="projectInputId">
					<ProjectCombobox
						v-model="selectedProjectId"
						:project-types="['modpack']"
						:exclude-project-ids="[currentProjectId]"
						:placeholder="formatMessage(messages.selectModpack)"
						:search-placeholder="formatMessage(messages.searchModpacks)"
					:search-input-id="projectInputId"
						:loading-message="formatMessage(commonMessages.loadingLabel)"
						:no-results-message="formatMessage(messages.noResults)"
						include-user-unlisted-projects
						:user-id="auth?.user?.id"
					/>
				</Field>

				<Field
					v-if="selectedProjectId"
					:label="formatMessage(messages.version)"
					:input-id="versionInputId"
				>
					<Combobox
						v-model="selectedVersionId"
						:placeholder="formatMessage(messages.selectVersion)"
						:options="versionOptions"
						:searchable="true"
					:search-input-id="versionInputId"
						:search-placeholder="formatMessage(messages.searchVersions)"
						:no-options-message="
							formatMessage(versionsLoading ? commonMessages.loadingLabel : messages.noResults)
						"
					/>
				</Field>
			</div>
			<div v-if="selectedVersion" class="flex flex-col gap-4 rounded-2xl bg-surface-2 p-4">
				<div class="flex items-center justify-between">
					<div class="text-secondary">{{ formatMessage(messages.gameVersion) }}</div>
					<div class="flex flex-wrap gap-1">
						<TagItem v-for="gv in selectedVersion.game_versions" :key="gv">
							{{ gv }}
						</TagItem>
					</div>
				</div>

				<div
					v-if="selectedVersion.mrpack_loaders?.length"
					class="flex items-center justify-between"
				>
					<div class="text-secondary">
						{{
							formatMessage(projectCompatibilityMessages.platformsPlural, {
								count: selectedVersion.mrpack_loaders.length,
							})
						}}
					</div>
					<div class="flex flex-wrap gap-1">
						<TagItem
							v-for="loader in selectedVersion.mrpack_loaders"
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
	</div>
</template>

<script setup lang="ts">
import { getLoaderIcon } from '@modrinth/assets'
import {
	Combobox,
	commonMessages,
	Field,
	FormattedTag,
	injectModrinthClient,
	injectNotificationManager,
	ProjectCombobox,
	projectCompatibilityMessages,
	TagItem,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref, useId, watch } from 'vue'

import { injectServerCompatibilityContext } from '~/providers/manage-server-compatibility-modal'

import DataLossWarningBanner from '../DataLossWarningBanner.vue'
import { compatibilityMessages as messages } from '../messages'

const { formatMessage } = useVIntl()

const { project, selectedProjectId, selectedVersionId } = injectServerCompatibilityContext()
const currentProjectId = computed(() => project.value?.id)
const { labrinth } = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const projectInputId = useId()
const versionInputId = useId()
const auth = await useAuth()

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
				title: formatMessage(messages.failedToLoadVersions),
				text: err.data?.description || String(error),
				type: 'error',
			})
		} finally {
			versionsLoading.value = false
		}
	},
	{ immediate: true },
)
</script>
