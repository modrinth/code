<template>
	<Transition name="fade" mode="out-in">
		<div
			v-if="!contentData"
			key="loading"
			class="flex min-h-[50vh] w-full flex-col items-center justify-center gap-2 text-center text-secondary"
		>
			<SpinnerIcon class="animate-spin" />
			Loading {{ type.toLowerCase() }}s...
		</div>

		<div
			v-else-if="error"
			key="error"
			class="flex w-full flex-col items-center justify-center gap-4 p-4"
		>
			<div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
				<div class="flex flex-col items-center text-center">
					<div class="flex flex-col items-center gap-4">
						<div class="grid place-content-center rounded-full bg-bg-orange p-4">
							<IssuesIcon class="size-12 text-orange" />
						</div>
						<h1 class="m-0 mb-2 w-fit text-4xl font-bold">Failed to load content</h1>
					</div>
					<p class="text-lg text-secondary">
						We couldn't load your server's {{ type.toLowerCase() }}s. Here's what went wrong:
					</p>
					<p>
						<span class="break-all font-mono">{{ error.message }}</span>
					</p>
					<ButtonStyled size="large" color="brand" @click="() => refetch()">
						<button class="mt-6 !w-full">Retry</button>
					</ButtonStyled>
				</div>
			</div>
		</div>

		<div v-else key="content" class="relative isolate flex h-full w-full flex-col">
			<div ref="pyroContentSentinel" class="sentinel" data-pyro-content-sentinel />
			<div class="relative flex h-full w-full flex-col">
				<div class="sticky top-0 z-20 -mt-3 flex items-center justify-between bg-bg py-3">
					<div class="flex w-full flex-col-reverse items-center gap-2 sm:flex-row">
						<div class="flex w-full items-center gap-2">
							<div class="relative flex-1 text-sm">
								<label class="sr-only" for="search">Search</label>
								<SearchIcon
									class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2"
									aria-hidden="true"
								/>
								<input
									id="search"
									v-model="searchInput"
									class="!h-9 !min-h-0 w-full border-[1px] border-solid border-button-border pl-9"
									type="search"
									name="search"
									autocomplete="off"
									:placeholder="`Search ${contentData?.length ?? 0} ${type.toLowerCase()}s...`"
									@input="debouncedSearch"
								/>
							</div>
							<ButtonStyled>
								<OverflowMenu
									:options="[
										{ id: 'all', action: () => (filterMethod = 'all') },
										{ id: 'enabled', action: () => (filterMethod = 'enabled') },
										{ id: 'disabled', action: () => (filterMethod = 'disabled') },
									]"
								>
									<span class="hidden whitespace-pre sm:block">
										{{ filterMethodLabel }}
									</span>
									<FilterIcon aria-hidden="true" />
									<DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
									<template #all> All {{ type.toLowerCase() }}s </template>
									<template #enabled> Only enabled </template>
									<template #disabled> Only disabled </template>
								</OverflowMenu>
							</ButtonStyled>
						</div>
						<div v-if="hasMods" class="flex w-full items-center gap-2 sm:w-fit">
							<ButtonStyled>
								<button class="w-full text-nowrap sm:w-fit" @click="initiateFileUpload">
									<FileIcon />
									Add file
								</button>
							</ButtonStyled>
							<ButtonStyled color="brand">
								<a
									class="w-full text-nowrap sm:w-fit"
									:href="`/discover/${type.toLowerCase()}s?sid=${serverId}`"
								>
									<PlusIcon />
									Add {{ type.toLowerCase() }}
								</a>
							</ButtonStyled>
						</div>
					</div>
				</div>

				<FileUploadDropdown
					ref="uploadDropdownRef"
					class="rounded-xl bg-bg-raised"
					:margin-bottom="16"
					:file-type="type"
					:current-path="`/${type.toLowerCase()}s`"
					:accepted-types="acceptedFileTypes"
					@upload-complete="() => refetch()"
				/>

				<FileUploadDragAndDrop
					v-if="server"
					class="relative min-h-[50vh]"
					overlay-class="rounded-xl border-2 border-dashed border-secondary"
					:type="type"
					@files-dropped="handleDroppedFiles"
				>
					<div v-if="hasFilteredMods" class="flex flex-col gap-2 transition-all">
						<div ref="listContainer" class="relative w-full">
							<div :style="{ position: 'relative', height: `${totalHeight}px` }">
								<div
									:style="{
										position: 'absolute',
										top: `${visibleTop}px`,
										width: '100%',
									}"
								>
									<TransitionGroup :name="hasInitiallyLoaded ? undefined : 'list'">
										<div
											v-for="(mod, index) in visibleItems.items"
											:key="getStableModKey(mod)"
											class="relative mb-2 flex w-full items-center justify-between rounded-xl bg-bg-raised"
											:class="mod.disabled ? 'bg-table-alternateRow text-secondary' : ''"
											:style="{
												height: '64px',
												transitionDelay: hasInitiallyLoaded
													? undefined
													: `${Math.min(index * 30, 300)}ms`,
											}"
										>
											<a
												:href="
													mod.project_id
														? `/project/${mod.project_id}/version/${mod.version_id}`
														: `files?path=${type.toLowerCase()}s`
												"
												class="flex min-w-0 flex-1 items-center gap-2 rounded-xl p-2"
												draggable="false"
											>
												<Avatar
													:src="mod.icon_url"
													size="sm"
													alt="Server Icon"
													:class="mod.disabled ? 'opacity-75 grayscale' : ''"
												/>
												<div class="flex min-w-0 flex-col gap-1">
													<span class="text-md flex min-w-0 items-center gap-2 font-bold">
														<span class="truncate text-contrast">{{ friendlyModName(mod) }}</span>
														<span
															v-if="mod.disabled"
															class="hidden rounded-full bg-button-bg p-1 px-2 text-xs text-contrast sm:block"
															>Disabled</span
														>
													</span>
													<div class="min-w-0 text-xs text-secondary">
														<span v-if="mod.owner" class="hidden sm:block">
															by {{ mod.owner }}
														</span>
														<span class="block font-semibold sm:hidden">
															{{ mod.version_number || `External ${type.toLowerCase()}` }}
														</span>
													</div>
												</div>
											</a>
											<div class="ml-2 hidden min-w-0 flex-1 flex-col text-sm sm:flex">
												<div class="truncate font-semibold text-contrast">
													<span v-tooltip="`${type} version`">{{
														mod.version_number || `External ${type.toLowerCase()}`
													}}</span>
												</div>
												<div class="truncate">
													<span v-tooltip="`${type} file name`">
														{{ mod.filename }}
													</span>
												</div>
											</div>
											<div
												class="flex items-center justify-end gap-2 pr-4 font-semibold text-contrast sm:min-w-44"
											>
												<ButtonStyled color="red" type="transparent">
													<button
														v-tooltip="`Delete ${type.toLowerCase()}`"
														:disabled="changingMods.has(getStableModKey(mod))"
														class="!hidden sm:!block"
														@click="removeMod(mod)"
													>
														<TrashIcon />
													</button>
												</ButtonStyled>
												<ButtonStyled type="transparent">
													<button
														v-tooltip="
															mod.project_id
																? `Edit ${type.toLowerCase()} version`
																: `External ${type.toLowerCase()}s cannot be edited`
														"
														:disabled="changingMods.has(getStableModKey(mod)) || !mod.project_id"
														class="!hidden sm:!block"
														@click="showVersionModal(mod)"
													>
														<template v-if="changingMods.has(getStableModKey(mod))">
															<SpinnerIcon class="animate-spin" />
														</template>
														<template v-else>
															<EditIcon />
														</template>
													</button>
												</ButtonStyled>

												<!-- Dropdown for mobile -->
												<div class="mr-2 flex items-center sm:hidden">
													<SpinnerIcon
														v-if="changingMods.has(getStableModKey(mod))"
														class="mr-2 h-5 w-5 animate-spin"
														style="color: var(--color-base)"
													/>
													<ButtonStyled v-else circular type="transparent">
														<OverflowMenu
															:options="[
																{
																	id: 'edit',
																	action: () => showVersionModal(mod),
																	shown: !!(
																		mod.project_id && !changingMods.has(getStableModKey(mod))
																	),
																},
																{
																	id: 'delete',
																	action: () => removeMod(mod),
																},
															]"
														>
															<MoreVerticalIcon aria-hidden="true" />
															<template #edit>
																<EditIcon class="h-5 w-5" />
																<span>Edit</span>
															</template>
															<template #delete>
																<TrashIcon class="h-5 w-5" />
																<span>Delete</span>
															</template>
														</OverflowMenu>
													</ButtonStyled>
												</div>

												<input
													:id="`toggle-${getStableModKey(mod)}`"
													:checked="!mod.disabled"
													:disabled="changingMods.has(getStableModKey(mod))"
													class="switch stylized-toggle"
													type="checkbox"
													@change="toggleMod(mod)"
												/>
											</div>
										</div>
									</TransitionGroup>
								</div>
							</div>
						</div>
					</div>

					<!-- no mods has platform -->
					<div
						v-else-if="server?.loader && server.loader.toLowerCase() !== 'vanilla'"
						class="mt-4 flex h-full flex-col items-center justify-center gap-4 text-center"
					>
						<div
							v-if="!hasFilteredMods && hasMods"
							class="mt-4 flex h-full flex-col items-center justify-center gap-4 text-center"
						>
							<SearchIcon class="size-24" />
							<p class="m-0 font-bold text-contrast">
								No {{ type.toLowerCase() }}s found for your query!
							</p>
							<p class="m-0">Try another query, or show everything.</p>
							<ButtonStyled>
								<button @click="showAll">
									<ListIcon />
									Show everything
								</button>
							</ButtonStyled>
						</div>
						<div
							v-else
							class="mt-4 flex h-full flex-col items-center justify-center gap-4 text-center"
						>
							<PackageClosedIcon class="size-24" />
							<p class="m-0 font-bold text-contrast">No {{ type.toLowerCase() }}s found!</p>
							<p class="m-0">
								Add some {{ type.toLowerCase() }}s to your server to manage them here.
							</p>
							<div class="flex flex-row items-center gap-4">
								<ButtonStyled type="outlined">
									<button class="w-full text-nowrap sm:w-fit" @click="initiateFileUpload">
										<FileIcon />
										Add file
									</button>
								</ButtonStyled>
								<ButtonStyled color="brand">
									<a
										class="w-full text-nowrap sm:w-fit"
										:href="`/discover/${type.toLowerCase()}s?sid=${serverId}`"
									>
										<PlusIcon />
										Add {{ type.toLowerCase() }}
									</a>
								</ButtonStyled>
							</div>
						</div>
					</div>

					<div
						v-else
						class="mt-4 flex h-full flex-col items-center justify-center gap-4 text-center"
					>
						<ServerLoaderIcon loader="Vanilla" class="size-24" />
						<p class="m-0 pt-3 font-bold text-contrast">Your server is running Vanilla Minecraft</p>
						<p class="m-0">
							Add content to your server by installing a modpack or choosing a different platform
							that supports {{ type }}s.
						</p>
						<div class="flex flex-row items-center gap-4">
							<ButtonStyled class="mt-8">
								<a :href="`/discover/modpacks?sid=${serverId}`">
									<CompassIcon />
									Find a modpack
								</a>
							</ButtonStyled>
							<div>or</div>
							<ButtonStyled class="mt-8">
								<a :href="`/hosting/manage/${serverId}/options/loader`">
									<WrenchIcon />
									Change platform
								</a>
							</ButtonStyled>
						</div>
					</div>
				</FileUploadDragAndDrop>
			</div>
		</div>
	</Transition>

	<ContentVersionEditModal
		v-if="server?.mc_version && server?.loader"
		ref="versionEditModalRef"
		:type="type"
		:loader="server.loader"
		:game-version="server.mc_version"
		:has-modpack="hasModpack"
		:server-id="serverId"
		:tags="props.tags"
		@change-version="handleVersionChange"
	/>
</template>

<script setup lang="ts">
import type { Archon, Labrinth } from '@modrinth/api-client'
import {
	CompassIcon,
	DropdownIcon,
	EditIcon,
	FileIcon,
	FilterIcon,
	IssuesIcon,
	ListIcon,
	MoreVerticalIcon,
	PackageClosedIcon,
	PlusIcon,
	SearchIcon,
	SpinnerIcon,
	TrashIcon,
	WrenchIcon,
} from '@modrinth/assets'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { useDebounceFn } from '@vueuse/core'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import Avatar from '../../../components/base/Avatar.vue'
import ButtonStyled from '../../../components/base/ButtonStyled.vue'
import OverflowMenu from '../../../components/base/OverflowMenu.vue'
import ContentVersionEditModal from '../../../components/servers/content/ContentVersionEditModal.vue'
import FileUploadDragAndDrop from '../../../components/servers/files/upload/FileUploadDragAndDrop.vue'
import FileUploadDropdown from '../../../components/servers/files/upload/FileUploadDropdown.vue'
import ServerLoaderIcon from '../../../components/servers/icons/LoaderIcon.vue'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '../../../providers'

export interface ContentPageTags {
	gameVersions: Labrinth.Tags.v2.GameVersion[]
	loaders: Labrinth.Tags.v2.Loader[]
}

const props = defineProps<{
	tags: ContentPageTags
}>()

type ContentItem = Archon.Content.v0.Mod

const ITEM_HEIGHT = 72
const BUFFER_SIZE = 5

// Modal state
const versionEditModalRef = ref<InstanceType<typeof ContentVersionEditModal>>()
const currentEditMod = ref<ContentItem | null>(null)

// Handle version change from modal
const handleVersionChange = async (versionId: string) => {
	const mod = currentEditMod.value
	if (!mod) return

	const modKey = getStableModKey(mod)
	changingMods.value.add(modKey)

	let installedNewVersion = false
	const oldFilename = mod.filename

	try {
		versionEditModalRef.value?.hide()

		// Step 1: Install new version
		await installMutation.mutateAsync({
			rinth_ids: { project_id: mod.project_id!, version_id: versionId },
			install_as: type.value.toLowerCase() as 'mod' | 'plugin',
		})
		installedNewVersion = true

		// Step 2: Remove old version
		await deleteMutation.mutateAsync({
			path: `/${type.value.toLowerCase()}s/${oldFilename}`,
			modKey,
		})

		// Step 3: Refresh content list
		await refetch()

		addNotification({
			text: `Successfully updated ${friendlyModName(mod)}`,
			type: 'success',
		})
	} catch (error) {
		const errorMsg = error instanceof Error ? error.message : String(error)

		if (installedNewVersion) {
			// New version installed but old version removal failed
			addNotification({
				text: `New version installed but couldn't remove old version. You may need to manually delete ${oldFilename}`,
				type: 'warning',
			})
		} else {
			// Installation failed
			addNotification({
				text: `Error changing mod version: ${errorMsg}`,
				type: 'error',
			})
		}

		console.error('Version change error:', error)
	} finally {
		changingMods.value.delete(modKey)
		currentEditMod.value = null
	}
}

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { server } = injectModrinthServerContext()

const route = useRoute()
const serverId = route.params.id as string

// Content type based on server loader
const type = computed(() => {
	const loader = server.value?.loader?.toLowerCase()
	return loader === 'paper' || loader === 'purpur' ? 'Plugin' : 'Mod'
})

// Check if server has a modpack
const hasModpack = computed(() => server.value?.upstream?.kind === 'modpack')

// Accepted file types for upload
const acceptedFileTypes = computed(() => {
	return type.value.toLowerCase() === 'plugin' ? ['.jar'] : ['.jar', '.zip']
})

// TanStack Query for content list
const contentQueryKey = computed(() => ['content', 'list', serverId])
const {
	data: contentData,
	error,
	refetch,
} = useQuery({
	queryKey: contentQueryKey,
	queryFn: () => client.archon.content_v0.list(serverId),
})

// Track mods currently being changed (toggle, delete, version change)
const changingMods = ref(new Set<string>())

// Delete mutation
const deleteMutation = useMutation({
	mutationFn: ({ path }: { path: string; modKey: string }) =>
		client.archon.content_v0.delete(serverId, { path }),
	onSuccess: () => {
		queryClient.invalidateQueries({ queryKey: contentQueryKey.value })
	},
	onError: (err, { modKey }) => {
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : 'Failed to remove mod',
		})
		changingMods.value.delete(modKey)
	},
})

// Toggle mutation (uses files API to rename)
const toggleMutation = useMutation({
	mutationFn: async ({ mod }: { mod: ContentItem; modKey: string }) => {
		const newFilename = mod.filename.endsWith('.disabled')
			? mod.filename.slice(0, -9)
			: `${mod.filename}.disabled`
		const folder = `${type.value.toLowerCase()}s`
		await client.kyros.files_v0.moveFileOrFolder(
			`/${folder}/${mod.filename}`,
			`/${folder}/${newFilename}`,
		)
		return { newFilename }
	},
	onSuccess: () => {
		queryClient.invalidateQueries({ queryKey: contentQueryKey.value })
	},
	onError: (_err, { mod, modKey }) => {
		addNotification({
			type: 'error',
			text: `Failed to toggle ${mod.name || mod.filename}`,
		})
		changingMods.value.delete(modKey)
	},
})

// Install mutation (for version changes)
const installMutation = useMutation({
	mutationFn: (req: Archon.Content.v0.InstallModRequest) =>
		client.archon.content_v0.install(serverId, req),
	onSuccess: () => {
		queryClient.invalidateQueries({ queryKey: contentQueryKey.value })
	},
})

// UI State
const searchInput = ref('')
const modSearchInput = ref('')
const filterMethod = ref('all')
const uploadDropdownRef = ref()
const listContainer = ref<HTMLElement | null>(null)
const windowScrollY = ref(0)
const windowHeight = ref(0)
const pyroContentSentinel = ref<HTMLElement | null>(null)

// Track initial load to only animate on first render
const hasInitiallyLoaded = ref(false)
watch(
	contentData,
	(data) => {
		if (data && !hasInitiallyLoaded.value) {
			setTimeout(() => {
				hasInitiallyLoaded.value = true
			}, 500)
		}
	},
	{ immediate: true },
)

// Debounced search
const debouncedSearch = useDebounceFn(() => {
	modSearchInput.value = searchInput.value

	if (pyroContentSentinel.value) {
		const sentinelRect = pyroContentSentinel.value.getBoundingClientRect()
		if (sentinelRect.top < 0 || sentinelRect.bottom > window.innerHeight) {
			pyroContentSentinel.value.scrollIntoView({ block: 'start' })
		}
	}
}, 300)

// Filter method label
const filterMethodLabel = computed(() => {
	switch (filterMethod.value) {
		case 'disabled':
			return 'Only disabled'
		case 'enabled':
			return 'Only enabled'
		default:
			return `All ${type.value.toLowerCase()}s`
	}
})

// Filtered and sorted mods
const filteredMods = computed(() => {
	if (!contentData.value) return []

	let mods = modSearchInput.value.trim()
		? contentData.value.filter(
				(mod) =>
					mod.name?.toLowerCase().includes(modSearchInput.value.toLowerCase()) ||
					mod.filename.toLowerCase().includes(modSearchInput.value.toLowerCase()),
			)
		: contentData.value

	switch (filterMethod.value) {
		case 'disabled':
			mods = mods.filter((mod) => mod.disabled)
			break
		case 'enabled':
			mods = mods.filter((mod) => !mod.disabled)
			break
	}

	return mods.sort((a, b) => friendlyModName(a).localeCompare(friendlyModName(b)))
})

const hasMods = computed(() => (contentData.value?.length ?? 0) > 0)
const hasFilteredMods = computed(() => filteredMods.value.length > 0)

// Virtualization
const totalHeight = computed(() => filteredMods.value.length * ITEM_HEIGHT)

const getVisibleRange = () => {
	if (!listContainer.value) return { start: 0, end: 0 }

	const containerTop = listContainer.value.getBoundingClientRect().top + window.scrollY
	const scrollTop = Math.max(0, windowScrollY.value - containerTop)

	const start = Math.floor(scrollTop / ITEM_HEIGHT)
	const visibleCount = Math.ceil(windowHeight.value / ITEM_HEIGHT)

	return {
		start: Math.max(0, start - BUFFER_SIZE),
		end: Math.min(filteredMods.value.length, start + visibleCount + BUFFER_SIZE * 2),
	}
}

const visibleTop = computed(() => {
	const range = getVisibleRange()
	return range.start * ITEM_HEIGHT
})

const visibleItems = computed(() => {
	const range = getVisibleRange()
	return {
		items: filteredMods.value.slice(
			Math.max(0, range.start),
			Math.min(filteredMods.value.length, range.end),
		),
	}
})

// Scroll handling
const handleScroll = () => {
	windowScrollY.value = window.scrollY
}

const handleResize = () => {
	windowHeight.value = window.innerHeight
}

onMounted(() => {
	windowHeight.value = window.innerHeight
	window.addEventListener('scroll', handleScroll, { passive: true })
	window.addEventListener('resize', handleResize, { passive: true })
	handleScroll()
})

onUnmounted(() => {
	window.removeEventListener('scroll', handleScroll)
	window.removeEventListener('resize', handleResize)
})

// Helper functions
function friendlyModName(mod: ContentItem) {
	if (mod.name) return mod.name

	let cleanName = mod.filename.endsWith('.disabled') ? mod.filename.slice(0, -9) : mod.filename

	const lastDotIndex = cleanName.lastIndexOf('.')
	if (lastDotIndex !== -1) cleanName = cleanName.substring(0, lastDotIndex)
	return cleanName
}

function getStableModKey(mod: ContentItem): string {
	if (mod.project_id) {
		return `project-${mod.project_id}`
	}
	const baseFilename = mod.filename.endsWith('.disabled') ? mod.filename.slice(0, -9) : mod.filename
	return `file-${baseFilename}`
}

// Actions
async function toggleMod(mod: ContentItem) {
	const modKey = getStableModKey(mod)
	changingMods.value.add(modKey)

	toggleMutation.mutate(
		{ mod, modKey },
		{
			onSettled: () => {
				changingMods.value.delete(modKey)
			},
		},
	)
}

async function removeMod(mod: ContentItem) {
	const modKey = getStableModKey(mod)
	changingMods.value.add(modKey)

	deleteMutation.mutate(
		{ path: `/${type.value.toLowerCase()}s/${mod.filename}`, modKey },
		{
			onSettled: () => {
				changingMods.value.delete(modKey)
			},
		},
	)
}

function showVersionModal(mod: ContentItem) {
	if (invalidModal.value || !mod?.project_id || !mod?.filename) {
		const errmsg = invalidModal.value
			? 'Data required for changing mod version was not found.'
			: `${!mod?.project_id ? 'No mod project ID found' : 'No mod filename found'} for ${friendlyModName(mod)}`
		console.error(errmsg)
		addNotification({
			text: errmsg,
			type: 'error',
		})
		return
	}

	currentEditMod.value = mod
	versionEditModalRef.value?.show(mod)
}

const handleDroppedFiles = (files: File[]) => {
	files.forEach((file) => {
		uploadDropdownRef.value?.uploadFile(file)
	})
}

const initiateFileUpload = () => {
	const input = document.createElement('input')
	input.type = 'file'
	input.accept = acceptedFileTypes.value.join(',')
	input.multiple = true
	input.onchange = () => {
		if (input.files) {
			Array.from(input.files).forEach((file) => {
				uploadDropdownRef.value?.uploadFile(file)
			})
		}
	}
	input.click()
}

const showAll = () => {
	searchInput.value = ''
	modSearchInput.value = ''
	filterMethod.value = 'all'
}
</script>

<style scoped>
.sentinel {
	position: absolute;
	top: -1rem;
	left: 0;
	right: 0;
	height: 1px;
	visibility: hidden;
}

.stylized-toggle:checked::after {
	background: var(--color-accent-contrast) !important;
}

.fade-enter-active,
.fade-leave-active {
	transition:
		opacity 300ms ease-in-out,
		transform 300ms ease-in-out;
}

.fade-enter-from,
.fade-leave-to {
	opacity: 0;
	transform: scale(0.98);
}

.list-enter-active,
.list-leave-active {
	transition: all 200ms ease-in-out;
}

.list-enter-from {
	opacity: 0;
	transform: translateY(-10px);
}

.list-leave-to {
	opacity: 0;
	transform: translateY(10px);
}

.list-move {
	transition: transform 200ms ease-in-out;
}
</style>
