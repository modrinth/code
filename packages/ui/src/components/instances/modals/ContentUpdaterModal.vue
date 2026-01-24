<template>
	<NewModal ref="modal" :header="header" :max-width="'90vw'" :width="'90vw'" no-padding>
		<div class="flex h-[643px] border-solid border-transparent border-[1px] border-b-surface-4">
			<div class="w-[272px] flex flex-col relative">
				<!-- Search input -->
				<div class="p-4 pb-2">
					<div class="iconified-input w-full border-solid border-[1px] border-surface-4 rounded-xl">
						<SearchIcon class="transition-colors" />
						<input
							v-model="searchQuery"
							type="text"
							placeholder="Search version..."
							class="!bg-transparent rounded-xl transition-colors"
						/>
					</div>
				</div>

				<!-- Version list (scrollable) -->
				<div class="flex-1 overflow-y-auto px-4 pb-16">
					<div class="flex flex-col gap-1.5">
						<button
							v-for="version in filteredVersions"
							:key="version.id"
							class="flex items-center h-10 px-4 py-2.5 rounded-xl border-none cursor-pointer transition-colors"
							:class="[
								selectedVersion?.id === version.id
									? 'bg-brand-highlight'
									: 'bg-transparent hover:bg-button-bg',
							]"
							@click="selectedVersion = version"
						>
							<div class="flex items-center justify-between w-full">
								<span
									v-tooltip="'v' + version.version_number"
									class="font-semibold text-contrast truncate"
								>
									v{{ version.version_number }}
								</span>
								<span
									class="px-2.5 py-0.5 rounded-full text-sm font-medium flex items-center flex-shrink-0 border border-solid"
									:class="getBadgeClasses(version)"
								>
									{{ getBadgeLabel(version) }}
								</span>
							</div>
						</button>
					</div>
					<div v-if="filteredVersions.length === 0" class="p-4 text-center text-secondary text-sm">
						No versions found
					</div>
				</div>

				<!-- Bottom gradient + hide incompatible toggle (overlay) -->
				<div class="absolute bottom-0 left-0 right-0 pointer-events-none">
					<div class="h-14 bg-gradient-to-t from-bg-raised to-transparent" />
					<div class="bg-bg-raised pb-5 flex justify-center pointer-events-auto">
						<ButtonStyled type="transparent" :circular="true">
							<button
								class="flex items-center gap-1.5"
								@click="hideIncompatible = !hideIncompatible"
							>
								<EyeIcon v-if="hideIncompatible" class="h-6 w-6" />
								<EyeOffIcon v-else class="h-6 w-6" />
								<span class="font-medium"
									>{{ hideIncompatible ? 'Show' : 'Hide' }} incompatible</span
								>
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>

			<div class="w-px bg-divider" />

			<div class="flex-1 flex flex-col min-w-0 relative">
				<template v-if="selectedVersion">
					<div class="bg-bg p-4">
						<div class="flex flex-col gap-1.5">
							<div class="flex items-center justify-between">
								<div class="flex items-center gap-1.5">
									<span class="font-semibold text-xl text-contrast">
										v{{ selectedVersion.version_number }}
									</span>
									<span
										class="px-2.5 py-0.5 rounded-full text-sm font-medium flex items-center flex-shrink-0 border border-solid"
										:class="getBadgeClasses(selectedVersion)"
									>
										{{ getBadgeLabel(selectedVersion) }}
									</span>
								</div>
								<span class="font-medium text-primary">
									{{ formatLongDate(selectedVersion.date_published) }}
								</span>
							</div>
							<div class="flex items-center gap-2">
								<div class="flex items-center gap-2 rounded-xl">
									<FileTextIcon class="h-6 w-6 text-primary" />
									<span class="font-medium text-primary">Changelog</span>
								</div>
								<span class="w-1.5 h-1.5 rounded-full bg-divider" />
								<span class="font-medium text-primary">
									{{ formatLoaderGameVersion(selectedVersion) }}
								</span>
							</div>
						</div>
					</div>

					<!-- Divider -->
					<div class="h-px bg-divider" />

					<!-- Changelog content -->
					<div class="flex-1 bg-bg p-4 overflow-y-auto">
						<div
							v-if="selectedVersion.changelog"
							class="changelog-body"
							v-html="renderHighlightedString(selectedVersion.changelog)"
						/>
						<div v-else class="text-secondary italic">No changelog provided for this version.</div>
					</div>

					<!-- Bottom gradient -->
					<div
						class="absolute bottom-0 left-0 right-0 h-14 bg-gradient-to-t from-bg to-transparent pointer-events-none"
					/>
				</template>
				<div v-else class="flex-1 flex items-center justify-center text-secondary bg-bg">
					Select a version to view its changelog
				</div>
			</div>
		</div>

		<template #actions>
			<div class="w-full flex flex-row gap-2 justify-end">
				<ButtonStyled type="outlined">
					<button class="!border-[1px] !border-surface-4" @click="handleCancel">
						<XIcon />
						Cancel
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button
						:disabled="!selectedVersion || selectedVersion.id === currentVersionId"
						@click="handleUpdate"
					>
						<DownloadIcon />
						Update to v{{ selectedVersion?.version_number ?? '...' }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	DownloadIcon,
	EyeIcon,
	EyeOffIcon,
	FileTextIcon,
	SearchIcon,
	XIcon,
} from '@modrinth/assets'
import { capitalizeString, renderHighlightedString } from '@modrinth/utils'
import { computed, ref } from 'vue'

import ButtonStyled from '../../base/ButtonStyled.vue'
import NewModal from '../../modal/NewModal.vue'

const props = withDefaults(
	defineProps<{
		versions: Labrinth.Versions.v2.Version[]
		currentGameVersion: string
		currentLoader: string
		currentVersionId: string
		header?: string
	}>(),
	{
		header: 'Update version',
	},
)

const emit = defineEmits<{
	update: [version: Labrinth.Versions.v2.Version]
	cancel: []
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const searchQuery = ref('')
const hideIncompatible = ref(true)
const selectedVersion = ref<Labrinth.Versions.v2.Version | null>(null)

function isVersionCompatible(version: Labrinth.Versions.v2.Version): boolean {
	const hasGameVersion = version.game_versions.includes(props.currentGameVersion)
	const hasLoader = version.loaders.some(
		(loader) => loader.toLowerCase() === props.currentLoader.toLowerCase(),
	)
	return hasGameVersion && hasLoader
}

const filteredVersions = computed(() => {
	let versions = [...props.versions]

	// Filter by search query
	if (searchQuery.value) {
		const query = searchQuery.value.toLowerCase()
		versions = versions.filter(
			(v) => v.name.toLowerCase().includes(query) || v.version_number.toLowerCase().includes(query),
		)
	}

	// Filter by compatibility
	if (hideIncompatible.value) {
		versions = versions.filter(isVersionCompatible)
	}

	return versions
})

function getBadgeLabel(version: Labrinth.Versions.v2.Version): string {
	if (version.id === props.currentVersionId) return 'Current'
	if (!isVersionCompatible(version)) return 'Incompatible'
	return capitalizeString(version.version_type)
}

function getBadgeClasses(version: Labrinth.Versions.v2.Version): string {
	// Current badge
	if (version.id === props.currentVersionId) {
		return 'bg-surface-4 border-surface-5 text-primary'
	}

	// Incompatible badge (takes precedence over version type)
	if (!isVersionCompatible(version)) {
		return 'bg-highlight-orange border-brand-orange text-brand-orange'
	}

	// Version type badges
	switch (version.version_type) {
		case 'release':
			return 'bg-highlight-green border-brand text-brand'
		case 'beta':
			return 'bg-highlight-blue border-brand-blue text-brand-blue'
		case 'alpha':
			return 'bg-highlight-purple border-brand-purple text-brand-purple'
		default:
			return 'bg-surface-4 border-surface-5 text-primary'
	}
}

function formatLongDate(dateString: string): string {
	return new Date(dateString).toLocaleDateString('en-US', {
		year: 'numeric',
		month: 'long',
		day: 'numeric',
	})
}

function formatLoaderGameVersion(version: Labrinth.Versions.v2.Version): string {
	const loader = capitalizeString(version.loaders[0] || '')
	const gameVersion = version.game_versions[0] || ''
	return `${loader} ${gameVersion}`
}

function handleUpdate() {
	if (selectedVersion.value) {
		emit('update', selectedVersion.value)
		hide()
	}
}

function handleCancel() {
	emit('cancel')
	hide()
}

function show(initialVersionId?: string) {
	searchQuery.value = ''
	hideIncompatible.value = true

	// Pre-select a version
	if (initialVersionId) {
		selectedVersion.value = props.versions.find((v) => v.id === initialVersionId) ?? null
	} else if (props.versions.length > 0) {
		// Default to first version if none specified
		selectedVersion.value = props.versions[0]
	} else {
		selectedVersion.value = null
	}

	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>

<style lang="scss" scoped>
:deep(.changelog-body) {
	line-height: 1.5;
	word-break: break-word;

	h1,
	h2,
	h3,
	h4,
	h5,
	h6 {
		margin: 0;
		font-weight: 600;
		color: var(--color-text-primary);
	}

	h3 {
		font-size: 1.125rem;
	}

	ul {
		padding-left: 1.5rem;
		margin: 0;
	}

	a {
		color: var(--color-link);

		&:hover,
		&:focus-visible {
			filter: brightness(1.2);
			text-decoration: underline;
		}
	}

	code {
		background-color: var(--color-bg);
		font-size: var(--font-size-sm);
		padding: 0.125rem 0.25rem;
		border-radius: 4px;
	}

	p {
		margin: 0;
		color: var(--color-text-default);
	}

	li {
		color: var(--color-text-default);
	}

	* + p {
		margin-top: 0.5rem;
	}

	h3 + * {
		margin-top: 0.25rem;
	}

	* + h3 {
		margin-top: 1.5rem;
	}

	* + li {
		margin-top: 0.25rem;
	}

	li ul li {
		margin-top: 0.25rem;
	}

	img {
		max-width: 100%;
		border-radius: var(--radius-md);
	}
}
</style>
