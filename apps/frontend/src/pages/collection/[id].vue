<template>
	<ConfirmModal
		v-if="canEdit"
		ref="deleteModal"
		:title="formatMessage(messages.deleteModalTitle)"
		:description="formatMessage(messages.deleteModalDescription)"
		:has-to-type="false"
		:proceed-label="formatMessage(commonMessages.deleteLabel)"
		@proceed="deleteCollection()"
	/>
	<NewModal v-if="canEdit" ref="editModal" :header="formatMessage(messages.editingCollection)">
		<div class="flex w-[30rem] flex-col gap-3">
			<div class="flow-root">
				<div class="group relative float-end ml-4">
					<OverflowMenu
						v-tooltip="formatMessage(messages.editIconButton)"
						class="m-0 cursor-pointer appearance-none border-none bg-transparent p-0 transition-transform group-active:scale-95"
						:options="[
							{
								id: 'select',
								action: () => {
									const input = iconInputRef?.$el?.querySelector('input')
									input?.click()
								},
							},
							{
								id: 'remove',
								color: 'danger',
								action: () => {
									deletedIcon = true
									previewImage = null
								},
								shown: !deletedIcon && (previewImage || collection.icon_url),
							},
						]"
					>
						<Avatar
							:src="deletedIcon ? null : previewImage ? previewImage : collection.icon_url"
							size="108px"
							class="!border-4 group-hover:brightness-75"
							no-shadow
						/>
						<div class="absolute right-0 top-0 m-2">
							<div
								class="m-0 flex aspect-square items-center justify-center rounded-full border-[1px] border-solid border-button-border bg-button-bg p-2 text-primary"
							>
								<EditIcon aria-hidden="true" class="h-4 w-4 text-primary" />
							</div>
						</div>
						<template #select>
							<UploadIcon />
							{{
								previewImage || collection.icon_url
									? formatMessage(messages.replaceIcon)
									: formatMessage(messages.selectIcon)
							}}
						</template>
						<template #remove>
							<XIcon />
							{{ formatMessage(messages.removeIconButton) }}
						</template>
					</OverflowMenu>
					<FileInput
						id="collection-icon-input"
						ref="iconInputRef"
						:max-size="262144"
						:show-icon="false"
						accept="image/png,image/jpeg,image/gif,image/webp"
						class="hidden"
						aria-label="Upload icon"
						@change="showPreviewImage"
					/>
				</div>
				<div class="overflow-hidden">
					<label class="mb-2 block text-lg font-semibold text-contrast" for="collection-title">
						{{ formatMessage(commonMessages.titleLabel) }}
					</label>
					<input
						id="collection-title"
						v-model="current.name"
						maxlength="255"
						type="text"
						autocomplete="off"
						class="w-full"
					/>
				</div>
				<label
					class="mb-2 mt-4 block text-lg font-semibold text-contrast"
					for="collection-description"
				>
					{{ formatMessage(commonMessages.descriptionLabel) }}
				</label>
				<div class="textarea-wrapper h-24">
					<textarea id="collection-description" v-model="current.description" maxlength="255" />
				</div>
				<label for="visibility" class="mb-2 mt-4 block text-lg font-semibold text-contrast">
					{{ formatMessage(commonMessages.visibilityLabel) }}
				</label>
				<RadioButtons v-model="current.status" :items="['listed', 'unlisted', 'private']">
					<template #default="{ item }">
						<span class="flex items-center gap-1">
							{{
								item === 'listed'
									? formatMessage(commonMessages.publicLabel)
									: formatMessage(commonMessages[`${item}Label`])
							}}
						</span>
					</template>
				</RadioButtons>
			</div>
			<div class="flex justify-end gap-2">
				<ButtonStyled class="w-24">
					<button @click="() => editModal?.hide()">
						<XIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand" class="w-36">
					<button :disabled="saving" @click="save()">
						<SpinnerIcon v-if="saving" class="animate-spin" aria-hidden="true" />
						<SaveIcon v-else aria-hidden="true" />
						{{
							saving
								? formatMessage(commonMessages.savingButton)
								: formatMessage(commonMessages.saveButton)
						}}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
	<NormalPage :sidebar="cosmetics.leftContentLayout ? 'left' : 'right'">
		<template #header>
			<div class="flex flex-col gap-6">
				<ClientOnly>
					<nuxt-link
						v-if="returnLink"
						:to="returnLink.link"
						class="flex w-fit items-center gap-1 text-brand-blue hover:underline"
					>
						<ChevronLeftIcon aria-hidden="true" />
						{{ formatMessage(returnLink.message, { user: creator.username }) }}
					</nuxt-link>
				</ClientOnly>
				<div class="grid grid-cols-[auto_1fr] gap-4 sm:grid-cols-[auto_1fr_auto]">
					<Avatar :src="collection.icon_url" size="64px" />
					<div class="flex flex-col gap-3">
						<h1 class="heading-2xl">
							{{ collection.name }}
						</h1>
						<div class="flex items-center gap-2">
							<template v-if="canEdit || collection.status !== 'listed'">
								<div class="flex items-center gap-1">
									<template v-if="collection.status === 'listed'">
										<GlobeIcon aria-hidden="true" />
										{{ formatMessage(commonMessages.publicLabel) }}
									</template>
									<template v-else-if="collection.status === 'unlisted'">
										<LinkIcon aria-hidden="true" />
										{{ formatMessage(commonMessages.unlistedLabel) }}
									</template>
									<template v-else-if="collection.status === 'private'">
										<LockIcon aria-hidden="true" />
										{{ formatMessage(commonMessages.privateLabel) }}
									</template>
									<template v-else-if="collection.status === 'rejected'">
										<XIcon aria-hidden="true" />
										{{ formatMessage(commonMessages.rejectedLabel) }}
									</template>
								</div>
								<span class="text-secondary">•</span>
							</template>
							<span>
								<IntlFormatted
									:message-id="messages.projectsCountLabel"
									:values="{
										count: formatCompactNumber(projects.length || 0),
										type: formatMessage(
											commonProjectTypeSentenceMessages[
												projectTypes.length === 1 ? projectTypes[0] : 'project'
											],
											{ count: projects.length || 0 },
										),
									}"
								>
									<template #stat="{ children }">
										<span class="primary-stat__counter">
											<component :is="() => normalizeChildren(children)" />
										</span>
									</template>
								</IntlFormatted>
							</span>
						</div>
					</div>
					<div class="col-span-2 flex gap-2 sm:col-span-1">
						<template v-if="canEdit">
							<ButtonStyled>
								<button @click="openEditModal">
									<EditIcon aria-hidden="true" />
									{{ formatMessage(commonMessages.editButton) }}
								</button>
							</ButtonStyled>
							<ButtonStyled color="red" color-fill="text">
								<button @click="() => $refs.deleteModal.show()">
									<TrashIcon aria-hidden="true" />
									{{ formatMessage(commonMessages.deleteLabel) }}
								</button>
							</ButtonStyled>
						</template>
					</div>
				</div>
				<HorizontalRule />
			</div>
		</template>
		<template #sidebar>
			<SidebarCard v-if="collection.description" :title="formatMessage(messages.descriptionLabel)">
				<p class="m-0">{{ collection.description }}</p>
			</SidebarCard>
			<SidebarCard
				v-if="collection.id !== 'following'"
				:title="formatMessage(messages.curatedByLabel)"
			>
				<nuxt-link
					class="group flex w-fit items-center gap-2 leading-[1.2] text-primary"
					:to="`/user/${creator.username}`"
				>
					<Avatar :src="creator.avatar_url" :alt="creator.username" size="32px" circle />
					<div class="flex flex-col">
						<span
							class="grid w-full grid-cols-[1fr_auto] flex-nowrap items-center gap-1 group-hover:underline"
						>
							<span class="min-w-0 overflow-hidden truncate">{{ creator.username }}</span>
						</span>
					</div>
				</nuxt-link>
			</SidebarCard>
			<AdPlaceholder v-if="!auth.user" />
			<SidebarCard
				v-if="collection.id !== 'following'"
				:title="formatMessage(messages.detailsLabel)"
			>
				<div class="flex flex-col gap-2">
					<span
						v-tooltip="dayjs(collection.created).format('MMMM D, YYYY [at] h:mm A')"
						class="flex w-fit items-center gap-2"
					>
						<CalendarIcon aria-hidden="true" />
						{{
							formatMessage(messages.createdAtLabel, {
								ago: formatRelativeTime(collection.created),
							})
						}}
					</span>
					<span
						v-if="showUpdatedDate"
						v-tooltip="dayjs(collection.updated).format('MMMM D, YYYY [at] h:mm A')"
						class="flex w-fit items-center gap-2"
					>
						<UpdatedIcon aria-hidden="true" />
						{{
							formatMessage(messages.updatedAtLabel, {
								ago: formatRelativeTime(collection.updated),
							})
						}}
					</span>
				</div>
			</SidebarCard>
		</template>
		<NavTabs
			v-if="projects && projectTypes.length > 1"
			:links="[
				{
					label: formatMessage(commonMessages.allProjectType),
					href: `/collection/${collection.id}`,
				},
				...projectTypes.map((x) => {
					return {
						label: formatMessage(commonProjectTypeCategoryMessages[x]),
						href: `/collection/${collection.id}/${x}s`,
					}
				}),
			]"
		/>

		<div
			v-if="projects && projects?.length > 0"
			:class="'project-list display-mode--' + (cosmetics.searchDisplayMode.collection || 'list')"
		>
			<ProjectCard
				v-for="project in (route.params.projectType !== undefined
					? projects.filter(
							(x) =>
								x.project_type ===
								route.params.projectType.substr(0, route.params.projectType.length - 1),
						)
					: projects
				)
					.slice()
					.sort((a, b) => b.downloads - a.downloads)"
				:id="project.id"
				:key="project.id"
				:type="project.project_type"
				:categories="project.categories"
				:created-at="project.published"
				:updated-at="project.updated"
				:description="project.description"
				:downloads="project.downloads ? project.downloads.toString() : '0'"
				:follows="project.followers ? project.followers.toString() : '0'"
				:featured-image="project.gallery.find((element) => element.featured)?.url"
				:icon-url="project.icon_url"
				:name="project.title"
				:client-side="project.client_side"
				:server-side="project.server_side"
				:color="project.color"
				:show-updated-date="!canEdit && collection.id !== 'following'"
				:show-created-date="!canEdit && collection.id !== 'following'"
			>
				<button
					v-if="canEdit"
					class="iconified-button remove-btn"
					:disabled="removing"
					@click="() => removeProject(project)"
				>
					<SpinnerIcon v-if="removing" class="animate-spin" aria-hidden="true" />
					<XIcon v-else aria-hidden="true" />
					{{ formatMessage(messages.removeProjectButton) }}
				</button>
				<button
					v-if="collection.id === 'following'"
					class="iconified-button"
					@click="unfollowProject(project)"
				>
					<HeartMinusIcon aria-hidden="true" />
					{{ formatMessage(messages.unfollowProjectButton) }}
				</button>
			</ProjectCard>
		</div>
		<div v-else>
			<div class="mx-auto flex flex-col justify-center gap-8 p-6 text-center">
				<EmptyIllustration class="h-[120px] w-auto" />
				<div class="-mt-4 flex flex-col gap-4">
					<div class="flex flex-col items-center gap-1.5">
						<span class="text-lg text-contrast md:text-2xl">{{
							formatMessage(messages.noProjectsLabel)
						}}</span>
					</div>
					<ButtonStyled v-if="auth.user && auth.user.id === creator.id" color="brand">
						<nuxt-link class="mx-auto w-min" to="/discover/mods">
							<CompassIcon class="size-5" />
							Discover mods
						</nuxt-link>
					</ButtonStyled>
				</div>
			</div>
		</div>
	</NormalPage>
</template>

<script setup>
import {
	CalendarIcon,
	ChevronLeftIcon,
	CompassIcon,
	EditIcon,
	EmptyIllustration,
	GlobeIcon,
	HeartMinusIcon,
	LinkIcon,
	LockIcon,
	SaveIcon,
	SpinnerIcon,
	TrashIcon,
	UpdatedIcon,
	UploadIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	commonMessages,
	commonProjectTypeCategoryMessages,
	commonProjectTypeSentenceMessages,
	ConfirmModal,
	FileInput,
	HorizontalRule,
	injectModrinthClient,
	injectNotificationManager,
	NewModal,
	normalizeChildren,
	NormalPage,
	OverflowMenu,
	RadioButtons,
	SidebarCard,
	useRelativeTime,
	useSavable,
} from '@modrinth/ui'
import { isAdmin } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { IntlFormatted } from '@vintl/vintl/components'
import dayjs from 'dayjs'

import AdPlaceholder from '~/components/ui/AdPlaceholder.vue'
import NavTabs from '~/components/ui/NavTabs.vue'
import ProjectCard from '~/components/ui/ProjectCard.vue'

const { handleError } = injectNotificationManager()
const api = injectModrinthClient()
const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const formatCompactNumber = useCompactNumber()

const route = useNativeRoute()
const router = useRouter()
const auth = await useAuth()
const cosmetics = useCosmetics()

const messages = defineMessages({
	collectionDescription: {
		id: 'collection.description',
		defaultMessage: '{description} - View the collection {name} by {username} on Modrinth',
	},
	collectionTitle: {
		id: 'collection.title',
		defaultMessage: '{name} - Collection',
	},
	editIconButton: {
		id: 'collection.button.edit-icon',
		defaultMessage: 'Edit icon',
	},
	removeIconButton: {
		id: 'collection.button.remove-icon',
		defaultMessage: 'Remove icon',
	},
	selectIcon: {
		id: 'collection.button.select-icon',
		defaultMessage: 'Select icon',
	},
	replaceIcon: {
		id: 'collection.button.replace-icon',
		defaultMessage: 'Replace icon',
	},
	editingCollection: {
		id: 'collection.editing',
		defaultMessage: 'Editing collection',
	},
	createdAtLabel: {
		id: 'collection.label.created-at',
		defaultMessage: 'Created {ago}',
	},
	collectionNotFoundError: {
		id: 'collection.error.not-found',
		defaultMessage: 'Collection not found',
	},
	curatedByLabel: {
		id: 'collection.label.curated-by',
		defaultMessage: 'Curated by',
	},
	descriptionLabel: {
		id: 'collection.label.description',
		defaultMessage: 'Description',
	},
	detailsLabel: {
		id: 'collection.label.details',
		defaultMessage: 'Details',
	},
	deleteModalDescription: {
		id: 'collection.delete-modal.description',
		defaultMessage: 'This will permanently delete this collection. This action cannot be undone.',
	},
	deleteModalTitle: {
		id: 'collection.delete-modal.title',
		defaultMessage: 'Are you sure you want to delete this collection?',
	},
	followingCollectionDescription: {
		id: 'collection.description.following',
		defaultMessage: "Auto-generated collection of all the projects you're following.",
	},
	noProjectsLabel: {
		id: 'collection.label.no-projects',
		defaultMessage: 'No projects in collection yet',
	},
	projectsCountLabel: {
		id: 'collection.label.projects-count',
		defaultMessage:
			'{count, plural, =0 {No projects yet} one {<stat>{count}</stat> project} other {<stat>{count}</stat> {type}}}',
	},
	removeProjectButton: {
		id: 'collection.button.remove-project',
		defaultMessage: 'Remove project',
	},
	unfollowProjectButton: {
		id: 'collection.button.unfollow-project',
		defaultMessage: 'Unfollow project',
	},
	updatedAtLabel: {
		id: 'collection.label.updated-at',
		defaultMessage: 'Updated {ago}',
	},
})

const returnLink = computed(() => {
	const from = router.options?.history?.state?.back
	if (from?.startsWith('/dashboard/collections')) {
		return {
			link: from,
			message: defineMessage({
				id: 'collection.return-link.dashboard-collections',
				defaultMessage: 'Your collections',
			}),
		}
	} else if (from?.startsWith('/user/')) {
		return {
			link: from,
			message: defineMessage({
				id: 'collection.return-link.user',
				defaultMessage: `{user}'s profile`,
			}),
		}
	}
	return null
})

let collection, refreshCollection, creator, projects

try {
	if (route.params.id === 'following') {
		collection = ref({
			id: 'following',
			icon_url: 'https://cdn.modrinth.com/follow-collection.png',
			name: formatMessage(commonMessages.followedProjectsLabel),
			description: formatMessage(messages.followingCollectionDescription),
			status: 'private',
			user: auth.value.user.id,
			created: auth.value.user.created,
			updated: auth.value.user.created,
		})
		;[{ data: projects }] = await Promise.all([
			useAsyncData(
				`user/${auth.value.user.id}/follows`,
				() => useBaseFetch(`user/${auth.value.user.id}/follows`),
				{
					transform: (projects) => {
						for (const project of projects) {
							project.categories = project.categories.concat(project.loaders)
						}

						return projects
					},
				},
			),
		])
		creator = ref(auth.value.user)
		refreshCollection = async () => {}
	} else {
		const val = await useAsyncData(`collection/${route.params.id}`, () =>
			useBaseFetch(`collection/${route.params.id}`, { apiVersion: 3 }),
		)
		collection = val.data
		refreshCollection = val.refresh
		;[{ data: creator }, { data: projects }] = await Promise.all([
			await useAsyncData(`user/${collection.value.user}`, () =>
				useBaseFetch(`user/${collection.value.user}`),
			),
			await useAsyncData(
				`projects?ids=${encodeURIComponent(JSON.stringify(collection.value.projects))}]`,
				() =>
					useBaseFetch(
						`projects?ids=${encodeURIComponent(JSON.stringify(collection.value.projects))}`,
					),
				{
					transform: (projects) => {
						for (const project of projects) {
							project.categories = project.categories.concat(project.loaders)
						}

						return projects
					},
				},
			),
		])
	}
} catch (err) {
	console.error(err)
	throw createError({
		fatal: true,
		statusCode: 404,
		message: formatMessage(messages.collectionNotFoundError),
	})
}

if (!collection.value) {
	throw createError({
		fatal: true,
		statusCode: 404,
		message: formatMessage(messages.collectionNotFoundError),
	})
}

const title = computed(() =>
	formatMessage(messages.collectionTitle, { name: collection.value.name }),
)

useSeoMeta({
	title,
	description: () =>
		formatMessage(messages.collectionDescription, {
			name: collection.value.name,
			description: collection.value.description,
			username: creator.value.username,
		}),
	ogTitle: title,
	ogDescription: collection.value.description,
	ogImage: collection.value.icon_url ?? 'https://cdn.modrinth.com/placeholder.png',
	robots: collection.value.status === 'listed' ? 'all' : 'noindex',
})

const canEdit = computed(
	() =>
		auth.value.user &&
		(auth.value.user.id === collection.value.user || isAdmin(auth.value.user)) &&
		collection.value.id !== 'following',
)

const projectTypes = computed(() => {
	const projectSet = new Set(
		projects.value?.map((project) => project?.project_type).filter((x) => x !== undefined) || [],
	)
	projectSet.delete('project')
	return Array.from(projectSet)
})

const showUpdatedDate = computed(() => {
	if (!collection.value?.updated || !collection.value?.created) {
		return false
	}
	return dayjs(collection.value.updated).diff(dayjs(collection.value.created), 'minute') > 1
})

const editModal = ref(null)
const iconInputRef = ref(null)
const icon = ref(null)
const deletedIcon = ref(false)
const previewImage = ref(null)
const saving = ref(false)
const removing = ref(false)

const {
	saved,
	current,
	reset,
	save: saveCollection,
} = useSavable(
	() => ({
		name: collection.value.name,
		description: collection.value.description || null,
		status: collection.value.status,
	}),
	async (changes) => {
		saving.value = true
		startLoading()
		try {
			if (deletedIcon.value) {
				await api.labrinth.collections.deleteIcon(collection.value.id)
			} else if (icon.value) {
				const ext = icon.value?.type?.split('/').pop()
				if (!ext) throw new Error('Invalid file type')
				await api.labrinth.collections.editIcon(collection.value.id, icon.value, ext)
			}

			if (Object.keys(changes).length > 0) {
				await api.labrinth.collections.edit(collection.value.id, changes)
			}

			await refreshCollection().then(reset)

			icon.value = null
			deletedIcon.value = false
			previewImage.value = null

			editModal.value?.hide()
		} catch (err) {
			handleError(err)
		} finally {
			await initUserCollections()
			stopLoading()
			saving.value = false
		}
	},
)

async function unfollowProject(project) {
	await userFollowProject(project)
	projects.value = projects.value.filter((x) => x.id !== project.id)
}

async function removeProject(project) {
	removing.value = true
	startLoading()
	try {
		const currentProjects = collection.value.projects || []
		const updatedProjects = currentProjects.filter((id) => id !== project.id)

		await api.labrinth.collections.edit(collection.value.id, {
			new_projects: updatedProjects,
		})

		await refreshCollection()
		projects.value = projects.value.filter((x) => x.id !== project.id)
		await initUserCollections()
	} catch (err) {
		handleError(err)
	} finally {
		removing.value = false
		stopLoading()
	}
}

const save = async () => {
	const hasIconChanges = deletedIcon.value || icon.value
	const hasCollectionChanges = computed(() => {
		const keys = Object.keys(current.value)
		for (const key of keys) {
			if (saved.value[key] !== current.value[key]) {
				return true
			}
		}
		return false
	}).value

	if (!hasCollectionChanges && !hasIconChanges) {
		return
	}

	if (hasCollectionChanges) {
		saveCollection()
	} else {
		saving.value = true
		startLoading()
		try {
			if (deletedIcon.value) {
				await api.labrinth.collections.deleteIcon(collection.value.id)
			} else if (icon.value) {
				const ext = icon.value?.type?.split('/').pop()
				if (!ext) throw new Error('Invalid file type')
				await api.labrinth.collections.editIcon(collection.value.id, icon.value, ext)
			}

			await refreshCollection().then(reset)

			icon.value = null
			deletedIcon.value = false
			previewImage.value = null

			editModal.value?.hide()
		} catch (err) {
			handleError(err)
		} finally {
			await initUserCollections()
			stopLoading()
			saving.value = false
		}
	}
}

async function deleteCollection() {
	startLoading()
	try {
		await api.labrinth.collections.delete(collection.value.id)
		if (auth.value.user.id === collection.value.user) {
			await navigateTo('/dashboard/collections')
		} else {
			await navigateTo(`/user/${collection.value.user}/collections`)
		}
	} catch (err) {
		handleError(err)
	}
	await initUserCollections()
	stopLoading()
}

function showPreviewImage(files) {
	const reader = new FileReader()
	icon.value = files[0]
	deletedIcon.value = false
	reader.readAsDataURL(icon.value)
	reader.onload = (event) => {
		previewImage.value = event.target.result
	}
}

function openEditModal(event) {
	reset()
	icon.value = null
	deletedIcon.value = false
	previewImage.value = null
	editModal.value?.show(event)
}
</script>

<style scoped lang="scss">
.animated-dropdown {
	// Omorphia's dropdowns are harcoded in width, so we need to override that
	width: 100% !important;
}
</style>
