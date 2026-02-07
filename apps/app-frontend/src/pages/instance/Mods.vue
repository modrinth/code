<template>
	<ContentPageLayout>
		<template #modals>
			<ShareModalWrapper
				ref="shareModal"
				share-title="Sharing modpack content"
				share-text="Check out the projects I'm using in my modpack!"
				:open-in-new-tab="false"
			/>
			<ModpackContentModal
				ref="modpackContentModal"
				:modpack-name="linkedModpackProject?.title"
				:modpack-icon-url="linkedModpackProject?.icon_url ?? undefined"
			/>
			<ExportModal v-if="projects.length > 0" ref="exportModal" :instance="instance" />
			<ContentUpdaterModal
				v-if="updatingProject || updatingModpack"
				ref="contentUpdaterModal"
				:versions="updatingProjectVersions"
				:current-game-version="instance.game_version"
				:current-loader="instance.loader"
				:current-version-id="
					updatingModpack
						? (instance.linked_data?.version_id ?? '')
						: (updatingProject?.version?.id ?? '')
				"
				:is-app="true"
				:is-modpack="updatingModpack"
				:project-icon-url="
					updatingModpack ? linkedModpackProject?.icon_url : updatingProject?.project?.icon_url
				"
				:project-name="
					updatingModpack
						? (linkedModpackProject?.title ?? 'Modpack')
						: (updatingProject?.project?.title ?? updatingProject?.file_name)
				"
				:loading="loadingVersions"
				:loading-changelog="loadingChangelog"
				@update="handleModalUpdate"
				@version-select="handleVersionSelect"
				@version-hover="handleVersionHover"
			/>
		</template>
	</ContentPageLayout>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	type ContentItem,
	type ContentModpackCardCategory,
	type ContentModpackCardProject,
	type ContentModpackCardVersion,
	type ContentOwner,
	ContentUpdaterModal,
	injectNotificationManager,
	ModpackContentModal,
	type OverflowMenuOption,
	provideContentManager,
} from '@modrinth/ui'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import ExportModal from '@/components/ui/ExportModal.vue'
import ShareModalWrapper from '@/components/ui/modal/ShareModalWrapper.vue'
import { trackEvent } from '@/helpers/analytics'
import { get_project_versions, get_version } from '@/helpers/cache.js'
import { profile_listener } from '@/helpers/events.js'
import {
	add_project_from_path,
	edit,
	get,
	get_content_items,
	get_linked_modpack_content,
	get_linked_modpack_info,
	remove_project,
	toggle_disable_project,
	update_managed_modrinth_version,
	update_project,
} from '@/helpers/profile'
import { get_categories } from '@/helpers/tags.js'
import type { CacheBehaviour, GameInstance } from '@/helpers/types'
import { highlightModInProfile } from '@/helpers/utils.js'
import { installVersionDependencies } from '@/store/install'
import ContentPageLayout from '@modrinth/ui/src/components/instances/ContentPageLayout.vue'

const { handleError, addNotification } = injectNotificationManager()
const router = useRouter()

const props = defineProps<{
	instance: GameInstance
	versions: Labrinth.Versions.v2.Version[]
}>()

const loading = ref(true)
const projects = ref<ContentItem[]>([])

const linkedModpackProject = ref<ContentModpackCardProject | null>(null)
const linkedModpackVersion = ref<ContentModpackCardVersion | null>(null)
const linkedModpackOwner = ref<ContentOwner | null>(null)
const linkedModpackCategories = ref<ContentModpackCardCategory[]>([])
const linkedModpackHasUpdate = ref(false)
const linkedModpackUpdateVersionId = ref<string | null>(null)

const isModpackUpdating = ref(false)
const isInstanceBusy = computed(() => props.instance?.install_stage !== 'installed')
const isPackLocked = computed(() => props.instance?.linked_data?.locked ?? false)

const shareModal = ref<InstanceType<typeof ShareModalWrapper> | null>()
const exportModal = ref(null)
const contentUpdaterModal = ref<InstanceType<typeof ContentUpdaterModal> | null>()
const modpackContentModal = ref<InstanceType<typeof ModpackContentModal> | null>()

const updatingProject = ref<ContentItem | null>(null)
const updatingProjectVersions = ref<Labrinth.Versions.v2.Version[]>([])
const loadingVersions = ref(false)
const loadingChangelog = ref(false)
const updatingModpack = ref(false)

async function handleBrowseContent() {
	if (!props.instance) return
	await router.push({
		path: `/browse/${props.instance.loader === 'vanilla' ? 'resourcepack' : 'mod'}`,
		query: { i: props.instance.path },
	})
}

async function handleUploadFiles() {
	if (!props.instance) return
	const files = await open({ multiple: true })
	if (!files) return

	const addedFiles: string[] = []
	for (const file of files) {
		const path = (file as { path?: string }).path ?? file
		const fileName = typeof path === 'string' ? (path.split('/').pop() ?? path) : String(path)
		try {
			await add_project_from_path(props.instance.path, path)
			addedFiles.push(fileName)
		} catch (e) {
			handleError(e as Error)
		}
	}
	await initProjects()

	if (addedFiles.length > 0) {
		const names = addedFiles.map((f) => {
			const item = projects.value.find(
				(p) => p.file_name === f || p.file_name === f.replace('.zip', '.jar'),
			)
			return item?.project?.title ?? f
		})
		addNotification({
			type: 'success',
			title: 'Successfully uploaded',
			text: names.length === 1 ? `"${names[0]}" was added` : `${names.length} projects were added`,
		})
	}
}

async function toggleDisableMod(mod: ContentItem) {
	try {
		mod.file_path = await toggle_disable_project(props.instance.path, mod.file_path!)
		mod.enabled = !mod.enabled

		trackEvent('InstanceProjectDisable', {
			loader: props.instance.loader,
			game_version: props.instance.game_version,
			id: mod.project?.id,
			name: mod.project?.title ?? mod.file_name,
			project_type: mod.project_type,
			disabled: !mod.enabled,
		})
	} catch (err) {
		handleError(err as Error)
	}
}

async function removeMod(mod: ContentItem) {
	await remove_project(props.instance.path, mod.file_path!).catch(handleError)
	projects.value = projects.value.filter((x) => mod.file_path !== x.file_path)

	trackEvent('InstanceProjectRemove', {
		loader: props.instance.loader,
		game_version: props.instance.game_version,
		id: mod.project?.id,
		name: mod.project?.title ?? mod.file_name,
		project_type: mod.project_type,
	})
}

async function updateProject(mod: ContentItem) {
	try {
		const newPath = await update_project(props.instance.path, mod.file_path!)
		mod.file_path = newPath

		if (mod.update_version_id) {
			const versionData = await get_version(mod.update_version_id, 'must_revalidate').catch(
				handleError,
			)

			if (versionData) {
				const profile = await get(props.instance.path).catch(handleError)

				if (profile) {
					await installVersionDependencies(profile, versionData).catch(handleError)
				}
			}
		}

		mod.has_update = false
		if (mod.version && mod.update_version_id) {
			mod.version.id = mod.update_version_id
		}
		mod.update_version_id = null

		trackEvent('InstanceProjectUpdate', {
			loader: props.instance.loader,
			game_version: props.instance.game_version,
			id: mod.project?.id,
			name: mod.project?.title ?? mod.file_name,
			project_type: mod.project_type,
		})
	} catch (err) {
		handleError(err as Error)
	}
}

async function handleUpdate(id: string) {
	const item = projects.value.find((p) => p.file_name === id)
	if (!item?.has_update || !item.project?.id || !item.version?.id) return

	updatingProject.value = item
	updatingProjectVersions.value = []
	loadingVersions.value = true
	loadingChangelog.value = false

	await nextTick()

	contentUpdaterModal.value?.show(item.update_version_id ?? undefined)

	const versions = (await get_project_versions(item.project.id).catch((e) => {
		return handleError(e)
	})) as Labrinth.Versions.v2.Version[] | null

	loadingVersions.value = false

	if (!versions) return

	versions.sort(
		(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
	)

	updatingProjectVersions.value = versions
}

async function handleModpackContent() {
	if (!props.instance?.path) return

	modpackContentModal.value?.showLoading()

	const contentItems = await get_linked_modpack_content(props.instance.path).catch(handleError)

	if (contentItems) {
		modpackContentModal.value?.show(contentItems)
	} else {
		modpackContentModal.value?.hide()
	}
}

async function handleModpackUpdate() {
	if (!props.instance?.linked_data?.project_id) return

	updatingModpack.value = true
	updatingProject.value = null
	updatingProjectVersions.value = []
	loadingVersions.value = true
	loadingChangelog.value = false

	await nextTick()

	contentUpdaterModal.value?.show(props.instance?.linked_data?.version_id ?? undefined)

	const versions = (await get_project_versions(props.instance.linked_data.project_id).catch(
		handleError,
	)) as Labrinth.Versions.v2.Version[] | null

	loadingVersions.value = false

	if (!versions) return

	versions.sort(
		(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
	)

	updatingProjectVersions.value = versions
}

async function handleVersionSelect(version: Labrinth.Versions.v2.Version) {
	if (version.changelog) return

	loadingChangelog.value = true

	const fullVersion = (await get_version(version.id, 'must_revalidate').catch(
		handleError,
	)) as Labrinth.Versions.v2.Version

	loadingChangelog.value = false

	if (!fullVersion) return

	const index = updatingProjectVersions.value.findIndex((v) => v.id === version.id)
	if (index !== -1) {
		const newVersions = [...updatingProjectVersions.value]
		newVersions[index] = fullVersion
		updatingProjectVersions.value = newVersions
	}
}

async function handleVersionHover(version: Labrinth.Versions.v2.Version) {
	if (version.changelog) return
	const fullVersion = (await get_version(version.id).catch(
		() => null,
	)) as Labrinth.Versions.v2.Version | null
	if (!fullVersion) return
	const index = updatingProjectVersions.value.findIndex((v) => v.id === version.id)
	if (index !== -1) {
		const newVersions = [...updatingProjectVersions.value]
		newVersions[index] = fullVersion
		updatingProjectVersions.value = newVersions
	}
}

async function handleModalUpdate(selectedVersion: Labrinth.Versions.v2.Version) {
	if (updatingModpack.value) {
		if (!props.instance?.path) return

		isModpackUpdating.value = true
		try {
			await update_managed_modrinth_version(props.instance.path, selectedVersion.id)
			await initProjects()
		} finally {
			isModpackUpdating.value = false
			updatingModpack.value = false
			updatingProjectVersions.value = []
			loadingVersions.value = false
			loadingChangelog.value = false
		}
	} else if (updatingProject.value) {
		const mod = updatingProject.value

		mod.update_version_id = selectedVersion.id

		await updateProject(mod)

		updatingProject.value = null
		updatingProjectVersions.value = []
		loadingVersions.value = false
		loadingChangelog.value = false
	}
}

async function unpairProfile() {
	await edit(props.instance.path, {
		linked_data: null as unknown as undefined,
	})
	linkedModpackProject.value = null
	linkedModpackVersion.value = null
	linkedModpackOwner.value = null
	linkedModpackHasUpdate.value = false
	linkedModpackUpdateVersionId.value = null
	await initProjects()
}

async function handleShareItems(
	items: ContentItem[],
	format: 'names' | 'file-names' | 'urls' | 'markdown',
) {
	const source = items.length > 0 ? items : projects.value
	let text: string
	switch (format) {
		case 'names':
			text = source.map((x) => x.project?.title ?? x.file_name).join('\n')
			break
		case 'file-names':
			text = source.map((x) => x.file_name).join('\n')
			break
		case 'urls':
			text = source
				.filter((x) => x.project?.slug)
				.map((x) => `https://modrinth.com/${x.project_type}/${x.project?.slug}`)
				.join('\n')
			break
		case 'markdown':
			text = source
				.map((x) => {
					const name = x.project?.title ?? x.file_name
					if (x.project?.slug) {
						return `[${name}](https://modrinth.com/${x.project_type}/${x.project.slug})`
					}
					return name
				})
				.join('\n')
			break
	}
	await shareModal.value?.show(text)
}

function getOverflowOptions(item: ContentItem): OverflowMenuOption[] {
	const options: OverflowMenuOption[] = [
		{
			id: 'Show file',
			action: () => highlightModInProfile(props.instance.path, item.file_path),
		},
	]

	if (item.project?.slug) {
		options.push({
			id: 'Copy link',
			action: async () => {
				await navigator.clipboard.writeText(
					`https://modrinth.com/${item.project_type}/${item.project?.slug}`,
				)
			},
		})
	}

	return options
}

async function initProjects(cacheBehaviour?: CacheBehaviour) {
	if (!props.instance) return

	const [contentItems, modpackInfo, allCategories] = await Promise.all([
		get_content_items(props.instance.path, cacheBehaviour).catch(handleError),
		get_linked_modpack_info(props.instance.path, cacheBehaviour).catch(handleError),
		get_categories().catch(handleError),
	])

	if (!contentItems) {
		loading.value = false
		return
	}

	projects.value = contentItems

	if (modpackInfo) {
		linkedModpackProject.value = {
			...modpackInfo.project,
			slug: modpackInfo.project.slug ?? modpackInfo.project.id,
			icon_url: modpackInfo.project.icon_url ?? undefined,
		}
		linkedModpackVersion.value = {
			...modpackInfo.version,
			date_published: modpackInfo.version.date_published.toString(),
		}
		linkedModpackOwner.value = modpackInfo.owner
			? {
					...modpackInfo.owner,
					avatar_url: modpackInfo.owner.avatar_url ?? undefined,
				}
			: null

		linkedModpackHasUpdate.value = modpackInfo.has_update
		linkedModpackUpdateVersionId.value = modpackInfo.update_version_id

		if (allCategories && modpackInfo.project.categories) {
			const seen = new Set<string>()
			linkedModpackCategories.value = allCategories
				.filter((cat: { name: string }) => {
					if (modpackInfo.project.categories.includes(cat.name) && !seen.has(cat.name)) {
						seen.add(cat.name)
						return true
					}
					return false
				})
				.map((cat: { name: string }) => ({
					...cat,
					name: cat.name.charAt(0).toUpperCase() + cat.name.slice(1),
				}))
		} else {
			linkedModpackCategories.value = []
		}
	} else {
		linkedModpackProject.value = null
		linkedModpackVersion.value = null
		linkedModpackOwner.value = null
		linkedModpackCategories.value = []
		linkedModpackHasUpdate.value = false
		linkedModpackUpdateVersionId.value = null
	}

	loading.value = false
}

provideContentManager({
	items: projects,
	loading,
	error: ref(null),
	modpack: computed(() =>
		linkedModpackProject.value
			? {
					project: linkedModpackProject.value,
					projectLink: `/project/${linkedModpackProject.value.slug ?? linkedModpackProject.value.id}`,
					version: linkedModpackVersion.value ?? undefined,
					versionLink:
						linkedModpackProject.value && linkedModpackVersion.value
							? `/project/${linkedModpackProject.value.slug ?? linkedModpackProject.value.id}/version/${linkedModpackVersion.value.id}`
							: undefined,
					owner: linkedModpackOwner.value ?? undefined,
					categories: linkedModpackCategories.value,
					hasUpdate: linkedModpackHasUpdate.value,
					disabled: isModpackUpdating.value,
					disabledText: isModpackUpdating.value ? 'Updating...' : 'Installing...',
				}
			: null,
	),
	isPackLocked,
	isBusy: isInstanceBusy,
	getItemId: (item) => item.file_name,
	contentTypeLabel: ref('project'),
	toggleEnabled: toggleDisableMod,
	deleteItem: removeMod,
	refresh: () => initProjects('must_revalidate'),
	browse: handleBrowseContent,
	uploadFiles: handleUploadFiles,
	hasUpdateSupport: true,
	updateItem: handleUpdate,
	bulkUpdateItem: updateProject,
	updateModpack: handleModpackUpdate,
	viewModpackContent: handleModpackContent,
	unlinkModpack: unpairProfile,
	getOverflowOptions,
	shareItems: handleShareItems,
	mapToTableItem: (item) => ({
		id: item.file_name,
		project: item.project ?? {
			id: item.file_name,
			slug: null,
			title: item.file_name.replace('.disabled', ''),
			icon_url: null,
		},
		projectLink: item.project?.id ? `/project/${item.project.id}` : undefined,
		version: item.version ?? {
			id: item.file_name,
			version_number: 'Unknown',
			file_name: item.file_name,
		},
		versionLink:
			item.project?.id && item.version?.id
				? `/project/${item.project.id}/version/${item.version.id}`
				: undefined,
		owner: item.owner
			? { ...item.owner, link: `https://modrinth.com/${item.owner.type}/${item.owner.id}` }
			: undefined,
		enabled: item.enabled,
	}),
})

await initProjects()

const unlisten = await getCurrentWebview().onDragDropEvent(async (event) => {
	if (event.payload.type !== 'drop' || !props.instance) return

	for (const file of event.payload.paths) {
		if (file.endsWith('.mrpack')) continue
		await add_project_from_path(props.instance.path, file).catch(handleError)
	}
	await initProjects()
})

const unlistenProfiles = await profile_listener(
	async (event: { event: string; profile_path_id: string }) => {
		if (
			props.instance &&
			event.profile_path_id === props.instance.path &&
			event.event === 'synced' &&
			props.instance.install_stage !== 'pack_installing'
		) {
			await initProjects()
		}
	},
)

watch(
	() => props.instance?.install_stage,
	async (newStage, oldStage) => {
		if (oldStage !== 'installed' && newStage === 'installed') {
			await initProjects('must_revalidate')
		}
	},
)

onUnmounted(() => {
	unlisten()
	unlistenProfiles()
})
</script>
