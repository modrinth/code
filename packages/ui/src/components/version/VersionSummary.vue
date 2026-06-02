<template>
	<div
		class="grid grid-cols-[min-content_auto_min-content_min-content] items-center gap-2 rounded-2xl border-[1px] border-divider bg-bg p-2"
	>
		<VersionChannelIndicator :channel="version.version_type" />

		<div class="flex min-w-0 flex-col gap-1">
			<h1 class="my-0 truncate text-nowrap text-base font-extrabold leading-none text-contrast">
				{{ version.version_number }}
			</h1>

			<p class="m-0 truncate text-nowrap text-xs font-semibold text-secondary">
				{{ version.name }}
			</p>
		</div>

		<ButtonStyled color="brand">
			<a
				:href="downloadUrl"
				:download="primaryFilename"
				class="min-w-0"
				@click="emit('onDownload')"
			>
				<DownloadIcon aria-hidden="true" />
				Download
			</a>
		</ButtonStyled>

		<ButtonStyled circular>
			<a
				:href="`/project/${props.version.project_id}/version/${props.version.id}`"
				class="min-w-0"
				aria-label="View version"
			>
				<ExternalIcon aria-hidden="true" />
			</a>
		</ButtonStyled>

		<template v-if="resolvedDeps.length > 0">
			<div class="col-span-4 mt-1 border-t border-divider pt-2">
				<p class="m-0 mb-1.5 text-[10px] font-semibold uppercase tracking-widest text-secondary">
					Also required
				</p>

				<div class="flex flex-col gap-1">
					<a
						v-for="dep in resolvedDeps"
						:key="dep.slug"
						:href="`/project/${dep.slug}`"
						class="flex items-center gap-2 rounded-xl bg-bg-raised px-2 py-1.5 hover:brightness-90"
					>
						<img
							v-if="dep.icon_url"
							:src="dep.icon_url"
							:alt="dep.title"
							class="h-6 w-6 flex-shrink-0 rounded-lg object-cover"
						/>

						<div
							v-else
							class="h-6 w-6 flex-shrink-0 rounded-lg bg-bg"
						/>

						<span
							class="min-w-0 flex-1 truncate text-sm font-medium text-primary"
						>
							{{ dep.title }}
						</span>

						<ExternalIcon
							class="h-4 w-4 flex-shrink-0 text-secondary"
							aria-hidden="true"
						/>
					</a>
				</div>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import { DownloadIcon, ExternalIcon } from '@modrinth/assets'
import type { Version, VersionFile } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { injectModrinthClient } from '../../providers/api-client'
import { ButtonStyled, VersionChannelIndicator } from '../index'

export interface ResolvedDep {
	id: string
	title: string
	slug: string
	icon_url?: string | null
	downloadUrl?: string | null
}

const props = defineProps<{
	version: Version
	decorateDownloadUrl?: (url: string) => string
	requiredDeps?: ResolvedDep[]
}>()

const client = injectModrinthClient()

const depsNeedingUrls = computed(() =>
	(props.requiredDeps ?? [])
		.filter((d) => !d.downloadUrl)
		.map((d) => d.id),
)

const { data: fetchedUrls } = useQuery({
	queryKey: computed(() => [
		'dep-file-urls',
		[...depsNeedingUrls.value].sort(),
	]),
	queryFn: async () => {
		const map: Record<string, string | null> = {}

		await Promise.all(
			depsNeedingUrls.value.map(async (id) => {
				try {
					const versions =
						await client.labrinth.versions_v2.getProjectVersions(id, {
							include_changelog: false,
						})

					const file =
						versions[0]?.files.find((f) => f.primary) ??
						versions[0]?.files[0]

					map[id] = file?.url ?? null
				} catch {
					map[id] = null
				}
			}),
		)

		return map
	},
	enabled: computed(() => depsNeedingUrls.value.length > 0),
	staleTime: 60 * 60 * 1000,
})

const resolvedDeps = computed(() =>
	(props.requiredDeps ?? []).map((dep) => ({
		...dep,
		downloadUrl: dep.downloadUrl ?? fetchedUrls.value?.[dep.id] ?? null,
	})),
)

const primaryFile = computed<VersionFile>(
	() => props.version.files.find((x) => x.primary) || props.version.files[0],
)

const downloadUrl = computed(() => {
	const raw = primaryFile.value.url
	return props.decorateDownloadUrl ? props.decorateDownloadUrl(raw) : raw
})

const primaryFilename = computed(() => primaryFile.value.filename)

const emit = defineEmits<{
	onDownload: []
	onNavigate: [url: string]
}>()
</script>
