<template>
	<div class="flex flex-col">
		<router-link
			class="mb-4 flex w-fit items-center gap-2 rounded-lg px-2 py-0.5 pl-0 text-link"
			:to="buildProjectHref(`/project/${route.params.id}/versions`)"
		>
			<ChevronLeftIcon class="shrink-0" /> {{ formatMessage(messages.allVersions) }}
		</router-link>
		<VersionPage
			v-if="version"
			:version="version"
			:enrichment="enrichment"
			:enrichment-loading="enrichmentLoading"
			:members="members"
			:dependency-link-creator="createDependencyLink"
		>
			<template #headerActions>
				<ButtonStyled color="brand">
					<button
						:disabled="installing || (installed && installedVersion === version.id)"
						@click="() => version && install(version.id)"
					>
						<DownloadIcon v-if="!installed" />
						<SwapIcon v-else-if="installedVersion !== version.id" />
						<CheckIcon v-else />
						{{
							installing
								? formatMessage(messages.installing)
								: installed && installedVersion === version.id
									? formatMessage(commonMessages.installedLabel)
									: installed
										? formatMessage(commonMessages.switchToVersionButton)
										: formatMessage(commonMessages.installButton)
						}}
					</button>
				</ButtonStyled>
				<ButtonStyled type="outlined" circular>
					<OverflowMenu
						v-tooltip="formatMessage(commonMessages.moreOptionsButton)"
						:options="[
							{
								id: 'open-in-browser',
								link: `https://modrinth.com/${project.project_type}/${project.slug}/version/${version.id}`,
								external: true,
							},
							{
								id: 'report',
								color: 'red',
								hoverFilled: true,
								link: `https://modrinth.com/report?item=version&itemID=${version.id}`,
								external: true,
							},
						]"
						aria-label="More options"
					>
						<MoreVerticalIcon aria-hidden="true" />
						<template #open-in-browser>
							<ExternalIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.openInBrowserButton) }}
						</template>
						<template #report>
							<ReportIcon aria-hidden="true" /> {{ formatMessage(commonMessages.reportButton) }}
						</template>
					</OverflowMenu>
				</ButtonStyled>
			</template>
			<template #supplementaryResourceActions="{ file }">
				<ButtonStyled>
					<a :href="file.url" :download="file.filename" target="_blank">
						<DownloadIcon aria-hidden="true" />
						{{ formatMessage(messages.downloadInBrowser) }}
					</a>
				</ButtonStyled>
			</template>
		</VersionPage>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckIcon,
	ChevronLeftIcon,
	DownloadIcon,
	ExternalIcon,
	MoreVerticalIcon,
	ReportIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	defineMessages,
	type DependencyContext,
	OverflowMenu,
	useVIntl,
	VersionPage,
} from '@modrinth/ui'
import { ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import { SwapIcon } from '@/assets/icons'
import { get_project_many, get_version_many } from '@/helpers/cache.js'
import { useBreadcrumbs } from '@/store/breadcrumbs'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	allVersions: {
		id: 'app.project.version.all-versions',
		defaultMessage: 'All versions',
	},
	installing: {
		id: 'app.project.version.installing',
		defaultMessage: 'Installing',
	},
	downloadInBrowser: {
		id: 'app.project.version.download-in-browser',
		defaultMessage: 'Download in browser',
	},
})

const breadcrumbs = useBreadcrumbs()
const route = useRoute()

const props = defineProps<{
	project: Labrinth.Projects.v2.Project
	versions: Labrinth.Versions.v3.Version[]
	members: Labrinth.Projects.v3.TeamMember[]
	install: (version: string | null) => void
	installed: boolean
	installing: boolean
	installedVersion: string
}>()

const version = ref(props.versions.find((version) => version.id === route.params.version))
if (version.value) {
	breadcrumbs.setName('Version', version.value.name)
}

const enrichment = ref<Labrinth.Projects.v2.DependencyInfo | undefined>(undefined)
const enrichmentLoading = ref(false)

function buildProjectHref(path: string): string {
	const params = new URLSearchParams()
	for (const [key, val] of Object.entries(route.query)) {
		if (Array.isArray(val)) {
			for (const v of val) {
				if (v != null) params.append(key, v)
			}
		} else if (val) {
			params.append(key, String(val))
		}
	}
	const qs = params.toString()
	return qs ? `${path}?${qs}` : path
}

function createDependencyLink(context: DependencyContext): string | undefined {
	if (context.version) {
		return buildProjectHref(`/project/${context.version.project_id}/version/${context.version.id}`)
	}
	if (context.project) {
		return buildProjectHref(`/project/${context.project.id}`)
	}
	return undefined
}

async function refreshEnrichment() {
	if (!version.value) return

	const projectIds = new Set<string>()
	const versionIds = new Set<string>()
	for (const dependency of version.value.dependencies ?? []) {
		if (dependency.project_id) {
			projectIds.add(dependency.project_id)
		}
		if (dependency.version_id) {
			versionIds.add(dependency.version_id)
		}
	}

	if (projectIds.size === 0 && versionIds.size === 0) {
		enrichment.value = { projects: [], versions: [] }
		return
	}

	enrichmentLoading.value = true
	try {
		const versionResults = versionIds.size > 0 ? await get_version_many([...versionIds]) : []
		for (const dependencyVersion of versionResults ?? []) {
			if (dependencyVersion.project_id) {
				projectIds.add(dependencyVersion.project_id)
			}
		}
		const projectResults = projectIds.size > 0 ? await get_project_many([...projectIds]) : []
		enrichment.value = {
			projects: projectResults ?? [],
			versions: versionResults ?? [],
		}
	} finally {
		enrichmentLoading.value = false
	}
}

watch(
	() => props.versions,
	async () => {
		if (route.params.version) {
			version.value = props.versions.find((v) => v.id === route.params.version)
			if (version.value) {
				breadcrumbs.setName('Version', version.value.name)
			}
			await refreshEnrichment()
		}
	},
)

await refreshEnrichment()
</script>
