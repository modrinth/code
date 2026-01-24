<template>
	<div v-if="version" class="version-page">
		<CreateProjectVersionModal
			v-if="currentMember"
			ref="createProjectVersionModal"
			@save="handleVersionSaved"
		/>
		<ConfirmModal
			v-if="currentMember"
			ref="modal_confirm"
			title="Are you sure you want to delete this version?"
			description="This will remove this version forever (like really forever)."
			:has-to-type="false"
			proceed-label="Delete"
			@proceed="deleteVersion()"
		/>
		<Modal v-if="auth.user && currentMember" ref="modal_package_mod" header="Package data pack">
			<div class="modal-package-mod universal-labels">
				<div class="markdown-body">
					<p>
						Package your data pack as a mod. This will create a new version with support for the
						selected mod loaders. You will be redirected to the new version and can edit it to your
						liking.
					</p>
				</div>
				<label for="package-mod-loaders">
					<span class="label__title">Mod loaders</span>
					<span class="label__description">
						The mod loaders you would like to package your data pack for.
					</span>
				</label>
				<multiselect
					id="package-mod-loaders"
					v-model="packageLoaders"
					:options="['fabric', 'forge', 'quilt', 'neoforge']"
					:custom-label="(value: string) => value.charAt(0).toUpperCase() + value.slice(1)"
					:multiple="true"
					:searchable="false"
					:show-no-results="false"
					:show-labels="false"
					placeholder="Choose loaders..."
					open-direction="top"
				/>
				<div class="button-group">
					<ButtonStyled>
						<button @click="modal_package_mod?.hide()">
							<XIcon aria-hidden="true" />
							Cancel
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button @click="createDataPackVersionHandler">
							<RightArrowIcon aria-hidden="true" />
							Begin packaging data pack
						</button>
					</ButtonStyled>
				</div>
			</div>
		</Modal>
		<div class="version-page__title universal-card">
			<Breadcrumbs
				:current-title="version.name"
				:link-stack="[
					{
						href: getPreviousLink(),
						label: getPreviousLabel(),
					},
				]"
			/>
			<div class="version-header">
				<template v-if="isEditing">
					<input
						v-model="version.name"
						type="text"
						placeholder="Enter a version title..."
						maxlength="256"
					/>
				</template>
				<h2 :class="{ 'sr-only': isEditing }">
					{{ version.name }}
				</h2>
			</div>
			<div v-if="fieldErrors && showKnownErrors" class="known-errors">
				<ul>
					<li v-if="version.version_number === ''">Your version must have a version number.</li>
					<li v-if="version.game_versions.length === 0">
						Your version must have the supported Minecraft versions selected.
					</li>
					<li v-if="newFiles.length === 0 && version.files.length === 0 && !replaceFile">
						Your version must have a file uploaded.
					</li>
					<li v-if="version.loaders.length === 0 && project.project_type !== 'resourcepack'">
						Your version must have the supported mod loaders selected.
					</li>
				</ul>
			</div>
			<div v-if="isCreating" class="input-group">
				<ButtonStyled color="brand">
					<button :disabled="shouldPreventActions" @click="createVersion">
						<PlusIcon aria-hidden="true" />
						Create
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<nuxt-link
						v-if="auth.user"
						:to="`/${project.project_type}/${project.slug ? project.slug : project.id}/versions`"
					>
						<XIcon aria-hidden="true" />
						Cancel
					</nuxt-link>
				</ButtonStyled>
			</div>
			<div v-else-if="isEditing" class="input-group">
				<ButtonStyled color="brand">
					<button :disabled="shouldPreventActions" @click="saveEditedVersion">
						<SaveIcon aria-hidden="true" />
						Save
					</button>
				</ButtonStyled>
				<ButtonStyled v-if="usesFeaturedVersions">
					<button
						v-tooltip="
							`Featured versions are being phased out. If you're still using this for something in the API, seek an alternative soon.`
						"
						@click="version.featured = !version.featured"
					>
						<StarIcon aria-hidden="true" />
						<template v-if="!version.featured"> Feature version (deprecated)</template>
						<template v-else> Unfeature version (deprecated)</template>
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<nuxt-link
						v-if="currentMember"
						class="action"
						:to="`/${project.project_type}/${
							project.slug ? project.slug : project.id
						}/version/${encodeURI(version.displayUrlEnding)}`"
					>
						<XIcon aria-hidden="true" />
						Discard changes
					</nuxt-link>
				</ButtonStyled>
			</div>
			<div v-else class="input-group mt-2">
				<ButtonStyled v-if="primaryFile && !currentMember" color="brand">
					<a
						v-tooltip="primaryFile.filename + ' (' + formatBytes(primaryFile.size) + ')'"
						:href="primaryFile.url"
						@click="emit('onDownload')"
					>
						<DownloadIcon aria-hidden="true" />
						Download
					</a>
				</ButtonStyled>
				<ButtonStyled v-if="!auth.user">
					<nuxt-link to="/auth/sign-in">
						<ReportIcon aria-hidden="true" />
						Report
					</nuxt-link>
				</ButtonStyled>
				<ButtonStyled v-else-if="!currentMember">
					<button @click="() => reportVersion(version.id)">
						<ReportIcon aria-hidden="true" />
						Report
					</button>
				</ButtonStyled>
				<ButtonStyled v-if="currentMember">
					<button @click="handleOpenEditVersionModal(version.id, project.id, 'metadata')">
						<BoxIcon aria-hidden="true" />
						Edit metadata
					</button>
				</ButtonStyled>
				<ButtonStyled v-if="currentMember">
					<button @click="handleOpenEditVersionModal(version.id, project.id, 'add-details')">
						<InfoIcon aria-hidden="true" />
						Edit details
					</button>
				</ButtonStyled>
				<ButtonStyled v-if="currentMember">
					<button @click="handleOpenEditVersionModal(version.id, project.id, 'add-files')">
						<FileIcon aria-hidden="true" />
						Edit files
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<button
						v-if="
							currentMember &&
							version.loaders.some((x: string) => tags.loaderData.dataPackLoaders.includes(x))
						"
						@click="modal_package_mod?.show()"
					>
						<BoxIcon aria-hidden="true" />
						Package as mod
					</button>
				</ButtonStyled>
			</div>
		</div>
		<div class="version-page__changelog universal-card">
			<h3>Changelog</h3>
			<div
				class="markdown-body"
				v-html="
					version.changelog ? renderHighlightedString(version.changelog) : 'No changelog specified.'
				"
			/>
		</div>
		<div
			v-if="sortedDeps.length > 0 || (isEditing && project.project_type !== 'modpack')"
			class="version-page__dependencies universal-card"
		>
			<h3>Dependencies</h3>
			<div
				v-for="(dependency, index) in sortedDeps.filter((x) => !x.file_name)"
				:key="index"
				class="dependency"
				:class="{ 'button-transparent': !isEditing }"
				@click="!isEditing ? router.push(dependency.link) : {}"
			>
				<Avatar
					:src="dependency.project ? dependency.project.icon_url : null"
					alt="dependency-icon"
					size="sm"
				/>
				<nuxt-link v-if="!isEditing" :to="dependency.link" class="info">
					<span class="project-title">
						{{ dependency.project ? dependency.project.title : 'Unknown Project' }}
					</span>
					<span v-if="dependency.version" class="dep-type" :class="dependency.dependency_type">
						Version {{ dependency.version.version_number }} is
						{{ dependency.dependency_type }}
					</span>
					<span v-else class="dep-type" :class="dependency.dependency_type">
						{{ dependency.dependency_type }}
					</span>
				</nuxt-link>
				<div v-else class="info">
					<span class="project-title">
						{{ dependency.project ? dependency.project.title : 'Unknown Project' }}
					</span>
					<span v-if="dependency.version" class="dep-type" :class="dependency.dependency_type">
						Version {{ dependency.version.version_number }} is
						{{ dependency.dependency_type }}
					</span>
					<span v-else class="dep-type" :class="dependency.dependency_type">
						{{ dependency.dependency_type }}
					</span>
				</div>
				<ButtonStyled v-if="isEditing && project.project_type !== 'modpack'">
					<button @click="version.dependencies.splice(index, 1)">
						<TrashIcon aria-hidden="true" />
						Remove
					</button>
				</ButtonStyled>
			</div>
			<div
				v-for="(dependency, index) in sortedDeps.filter((x) => x.file_name)"
				:key="index"
				class="dependency"
			>
				<Avatar alt="dependency-icon" size="sm" />
				<div class="info">
					<span class="project-title">
						{{ dependency.file_name }}
					</span>
					<span class="dep-type" :class="dependency.dependency_type">Added via overrides</span>
				</div>
			</div>
		</div>
		<div class="version-page__files universal-card">
			<h3>Files</h3>
			<div
				v-for="file in version.files"
				:key="file.hashes.sha1"
				:class="{
					file: true,
					primary: primaryFile.hashes.sha1 === file.hashes.sha1,
				}"
			>
				<FileIcon aria-hidden="true" />
				<span class="filename">
					<strong>{{ file.filename }}</strong>
					<span class="file-size">({{ formatBytes(file.size) }})</span>
					<span v-if="primaryFile.hashes.sha1 === file.hashes.sha1" class="file-type">
						Primary
					</span>
					<span
						v-else-if="file.file_type === 'required-resource-pack' && !isEditing"
						class="file-type"
					>
						Required resource pack
					</span>
					<span
						v-else-if="file.file_type === 'optional-resource-pack' && !isEditing"
						class="file-type"
					>
						Optional resource pack
					</span>
				</span>
				<ButtonStyled>
					<a
						:href="file.url"
						class="raised-button"
						:title="`Download ${file.filename}`"
						tabindex="0"
					>
						<DownloadIcon aria-hidden="true" />
						Download
					</a>
				</ButtonStyled>
			</div>
		</div>
		<div class="version-page__metadata">
			<div class="universal-card full-width-inputs">
				<h3>Metadata</h3>
				<div>
					<h4>Release channel</h4>
					<Badge
						v-if="version.version_type === 'release'"
						class="value"
						type="release"
						color="green"
					/>
					<Badge
						v-else-if="version.version_type === 'beta'"
						class="value"
						type="beta"
						color="orange"
					/>
					<Badge
						v-else-if="version.version_type === 'alpha'"
						class="value"
						type="alpha"
						color="red"
					/>
				</div>
				<div>
					<h4>Version number</h4>
					<span>{{ version.version_number }}</span>
				</div>
				<div v-if="project.project_type !== 'resourcepack'">
					<h4>Loaders</h4>

					<Categories :categories="version.loaders" :type="project.project_type" />
				</div>
				<div>
					<h4>Game versions</h4>
					<span>{{ formatVersionDisplay(version.game_versions) }}</span>
				</div>
				<div v-if="!isEditing && environment">
					<h4>Environment</h4>
					<div class="flex items-center gap-1.5">
						<template v-if="(environment as any).icon">
							<component :is="(environment as any).icon" />
						</template>
						<span>
							{{ environment.title.defaultMessage }}
						</span>
					</div>
				</div>
				<div v-if="!isEditing">
					<h4>Downloads</h4>
					<span>{{ version.downloads }}</span>
				</div>
				<div v-if="!isEditing">
					<h4>Publication date</h4>
					<span>
						{{ $dayjs(version.date_published).format('MMMM D, YYYY [at] h:mm A') }}
					</span>
				</div>
				<div v-if="!isEditing && version.author">
					<h4>Publisher</h4>
					<div
						class="team-member columns button-transparent"
						@click="router.push('/user/' + version.author.user.username)"
					>
						<Avatar
							:src="version.author.avatar_url"
							:alt="version.author.user.username"
							size="sm"
							circle
						/>

						<div class="member-info">
							<nuxt-link :to="'/user/' + version.author.user.username" class="name">
								<p>
									{{ version.author.name }}
								</p>
							</nuxt-link>
							<p v-if="version.author.role" class="role">
								{{ version.author.role }}
							</p>
							<p v-else-if="version.author_id === 'GVFjtWTf'" class="role">Archivist</p>
						</div>
					</div>
				</div>
				<div v-if="!isEditing">
					<h4>Version ID</h4>
					<CopyCode :text="version.id" />
				</div>
				<div v-if="!isEditing && flags.developerMode">
					<h4>Maven coordinates</h4>
					<div class="maven-section">
						<CopyCode :text="`maven.modrinth:${project.id}:${version.id}`" />
					</div>
				</div>
			</div>
		</div>
	</div>
</template>
<script setup lang="ts">
import {
	BoxIcon,
	DownloadIcon,
	FileIcon,
	InfoIcon,
	PlusIcon,
	ReportIcon,
	RightArrowIcon,
	SaveIcon,
	StarIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Badge,
	ButtonStyled,
	ConfirmModal,
	CopyCode,
	ENVIRONMENTS_COPY,
	injectNotificationManager,
	injectProjectPageContext,
} from '@modrinth/ui'
import { formatBytes, renderHighlightedString } from '@modrinth/utils'
import { Multiselect } from 'vue-multiselect'

import Breadcrumbs from '~/components/ui/Breadcrumbs.vue'
import CreateProjectVersionModal from '~/components/ui/create-project-version/CreateProjectVersionModal.vue'
import Modal from '~/components/ui/Modal.vue'
import Categories from '~/components/ui/search/Categories.vue'
import { useImageUpload } from '~/composables/image-upload.ts'
import { inferVersionInfo } from '~/helpers/infer'
import { createDataPackVersion } from '~/helpers/package.js'
import { reportVersion } from '~/utils/report-helpers.ts'
const emit = defineEmits<{
	onDownload: []
}>()

// Composables
const data = useNuxtApp()
const route = useNativeRoute()
const router = useRouter()
const auth = await useAuth()
const tags = useGeneratedState()
const flags = useFeatureFlags()
const { addNotification } = injectNotificationManager()

// Helper for accessing nuxt app $formatVersion
const formatVersionDisplay = (versions: string[]) => (data as any).$formatVersion(versions)

// Get data from DI context
const {
	projectV2: project,
	currentMember,
	allMembers: members,
	versions: contextVersions,
	loadVersions,
	dependencies: contextDependencies,
	loadDependencies,
	refreshVersions,
	refreshProject,
} = injectProjectPageContext()

// Load versions and dependencies in parallel
await Promise.all([loadVersions(), loadDependencies()])

// Template refs
const createProjectVersionModal = useTemplateRef('createProjectVersionModal')
const modal_confirm = useTemplateRef('modal_confirm')
const modal_package_mod = useTemplateRef('modal_package_mod')

// Initial mode calculation
const path = route.name?.toString().split('-') ?? []
const initialMode = path[path.length - 1]

// Reactive state from data()
const _dependencyAddMode = ref('project')
const _newDependencyType = ref('required')
const newDependencyId = ref('')
const _showSnapshots = ref(false)
const newFiles = ref<File[]>([])
const deleteFiles = ref<string[]>([])
const newFileTypes = ref<Array<{ display: string; value: string } | null>>([])
const packageLoaders = ref(['forge', 'fabric', 'quilt', 'neoforge'])
const showKnownErrors = ref(false)
const shouldPreventActions = ref(false)
const uploadedImageIds = ref<string[]>([])

// File types constant
const fileTypes = ref([
	{
		display: 'Required resource pack',
		value: 'required-resource-pack',
	},
	{
		display: 'Optional resource pack',
		value: 'optional-resource-pack',
	},
])

// Mutable state initialized during setup
const isCreating = ref(false)
const isEditing = ref(false)
const version = ref<Record<string, any>>({})
const primaryFile = ref<Record<string, any>>({})
const alternateFile = ref<Record<string, any> | undefined>(undefined)
const replaceFile = ref<File | null>(null)
const oldFileTypes = ref<Array<{ display: string; value: string } | null>>([])

// Initialize version data
if (initialMode === 'edit') {
	isEditing.value = true
}

if (route.params.version === 'create') {
	isCreating.value = true
	isEditing.value = true

	version.value = {
		id: 'none',
		project_id: project.value.id,
		author_id: currentMember.value?.user.id,
		name: '',
		version_number: '',
		changelog: '',
		date_published: Date.now(),
		downloads: 0,
		version_type: 'release',
		files: [],
		dependencies: [],
		game_versions: [],
		loaders: [],
		featured: false,
	}

	// For navigation from versions page / upload file prompt
	if (import.meta.client && history.state && history.state.newPrimaryFile) {
		replaceFile.value = history.state.newPrimaryFile

		try {
			const inferredData = await inferVersionInfo(
				replaceFile.value!,
				project.value as any,
				tags.value.gameVersions,
			)

			version.value = {
				...version.value,
				...inferredData,
			}
		} catch (err) {
			console.error('Error parsing version file data', err)
		}
	}
} else if (route.params.version === 'latest') {
	let versionList = contextVersions.value ?? []
	if (route.query.loader) {
		versionList = versionList.filter((x: any) => x.loaders.includes(route.query.loader))
	}
	if (route.query.version) {
		versionList = versionList.filter((x: any) => x.game_versions.includes(route.query.version))
	}
	if (versionList.length === 0) {
		throw createError({
			fatal: true,
			statusCode: 404,
			message: 'No version matches the filters',
		})
	}
	version.value = versionList.reduce((a: any, b: any) =>
		a.date_published > b.date_published ? a : b,
	)
} else {
	let foundVersion = ((contextVersions.value ?? []) as any[]).find(
		(x: any) => x.id === route.params.version,
	)

	if (!foundVersion) {
		foundVersion = ((contextVersions.value ?? []) as any[]).find(
			(x: any) => x.displayUrlEnding === route.params.version,
		)
	}

	if (!foundVersion) {
		foundVersion = ((contextVersions.value ?? []) as any[]).find(
			(x: any) => x.version_number === route.params.version,
		)
	}

	if (foundVersion) {
		const versionV3 = (await useBaseFetch(
			`project/${project.value.id}/version/${route.params.version}`,
			{ apiVersion: 3 },
		)) as any
		if (versionV3) {
			foundVersion.environment = versionV3.environment
			foundVersion.changelog = versionV3.changelog
		}
		version.value = foundVersion
	}
}

if (!version.value || Object.keys(version.value).length === 0) {
	throw createError({
		fatal: true,
		statusCode: 404,
		message: 'Version not found',
	})
}

// Deep clone version to make it reactive and avoid mutating the original
version.value = JSON.parse(JSON.stringify(version.value))
primaryFile.value =
	version.value.files?.find((file: any) => file.primary) ?? version.value.files?.[0] ?? {}
alternateFile.value = version.value.files?.find(
	(file: any) => file.file_type && file.file_type.includes('resource-pack'),
)

// Process dependencies
const deps = contextDependencies.value ?? { projects: [], versions: [] }
for (const dependency of version.value.dependencies ?? []) {
	dependency.version = deps.versions.find((x: any) => x.id === dependency.version_id)

	if (dependency.version) {
		dependency.project = deps.projects.find((x: any) => x.id === dependency.version.project_id)
	}

	if (!dependency.project) {
		dependency.project = deps.projects.find((x: any) => x.id === dependency.project_id)
	}

	dependency.link = dependency.project
		? `/${dependency.project.project_type}/${dependency.project.slug ?? dependency.project.id}${
				dependency.version ? `/version/${encodeURI(dependency.version.version_number)}` : ''
			}`
		: ''
}

oldFileTypes.value = (version.value.files ?? []).map(
	(x: any) => fileTypes.value.find((y) => y.value === x.file_type) ?? null,
)

// Computed properties
const title = computed(
	() => `${isCreating.value ? 'Create Version' : version.value.name} - ${project.value.title}`,
)

const description = computed(
	() =>
		`Download ${project.value.title} ${
			version.value.version_number
		} on Modrinth. Supports ${(data as any).$formatVersion(version.value.game_versions)} ${(
			version.value.loaders ?? []
		)
			.map((x: string) => x.charAt(0).toUpperCase() + x.slice(1))
			.join(' & ')}. Published on ${data
			.$dayjs(version.value.date_published)
			.format('MMM D, YYYY')}. ${version.value.downloads} downloads.`,
)

const usesFeaturedVersions = computed(() =>
	(contextVersions.value ?? []).some((v: any) => v.featured),
)

const fieldErrors = computed(
	() =>
		version.value.version_number === '' ||
		(version.value.game_versions?.length ?? 0) === 0 ||
		((version.value.loaders?.length ?? 0) === 0 && project.value.project_type !== 'resourcepack') ||
		(newFiles.value.length === 0 && (version.value.files?.length ?? 0) === 0 && !replaceFile.value),
)

const sortedDeps = computed(() => {
	const order = ['required', 'optional', 'incompatible', 'embedded']
	return [...(version.value.dependencies ?? [])].sort(
		(a, b) => order.indexOf(a.dependency_type) - order.indexOf(b.dependency_type),
	)
})

const environment = computed(
	() => ENVIRONMENTS_COPY[version.value.environment as keyof typeof ENVIRONMENTS_COPY],
)

// SEO
useSeoMeta({
	title,
	description,
	ogTitle: title,
	ogDescription: description,
})

// Watch route changes
watch(
	() => route.path,
	() => {
		const routePath = route.name?.toString().split('-') ?? []
		const mode = routePath[routePath.length - 1]
		isEditing.value = mode === 'edit' || route.params.version === 'create'
	},
)

// Methods
function handleOpenEditVersionModal(versionId: string, projectId: string, stageId: string) {
	if (!currentMember.value) return
	createProjectVersionModal.value?.openEditVersionModal(versionId, projectId, stageId)
}

async function handleVersionSaved() {
	router.go(0) // reload page for new data
}

async function _onImageUpload(file: File) {
	const response = await useImageUpload(file, { context: 'version' })

	uploadedImageIds.value.push(response.id)
	uploadedImageIds.value = uploadedImageIds.value.slice(-10)

	return response.url
}

function getPreviousLink() {
	if (router.options.history.state.back) {
		if ((router.options.history.state.back as string).includes('/versions')) {
			return router.options.history.state.back as string
		}
	}
	return `/${project.value.project_type}/${project.value.slug ? project.value.slug : project.value.id}/versions`
}

function getPreviousLabel() {
	return router.options.history.state.back &&
		(router.options.history.state.back as string).endsWith('/versions')
		? 'Back to versions'
		: 'All versions'
}

async function _addDependency(
	dependencyAddModeParam: string,
	newDependencyIdParam: string,
	newDependencyTypeParam: string,
	hideErrors?: boolean,
) {
	try {
		if (dependencyAddModeParam === 'project') {
			const project = (await useBaseFetch(`project/${newDependencyIdParam}`)) as any

			if (version.value.dependencies.some((dep: any) => project.id === dep.project_id)) {
				addNotification({
					title: 'Dependency already added',
					text: 'You cannot add the same dependency twice.',
					type: 'error',
				})
			} else {
				version.value.dependencies.push({
					project,
					project_id: project.id,
					dependency_type: newDependencyTypeParam,
					link: `/${project.project_type}/${project.slug ?? project.id}`,
				})
			}
		} else if (dependencyAddModeParam === 'version') {
			const versionData = (await useBaseFetch(`version/${newDependencyIdParam}`)) as any
			const project = (await useBaseFetch(`project/${versionData.project_id}`)) as any

			if (version.value.dependencies.some((dep: any) => versionData.id === dep.version_id)) {
				addNotification({
					title: 'Dependency already added',
					text: 'You cannot add the same dependency twice.',
					type: 'error',
				})
			} else {
				version.value.dependencies.push({
					version: versionData,
					project,
					version_id: versionData.id,
					project_id: project.id,
					dependency_type: newDependencyTypeParam,
					link: `/${project.project_type}/${project.slug ?? project.id}/version/${encodeURI(
						versionData.version_number,
					)}`,
				})
			}
		}

		newDependencyId.value = ''
	} catch {
		if (!hideErrors) {
			addNotification({
				title: 'Invalid Dependency',
				text: 'The specified dependency could not be found',
				type: 'error',
			})
		}
	}
}

async function saveEditedVersion() {
	startLoading()

	if (fieldErrors.value) {
		showKnownErrors.value = true
		stopLoading()
		return
	}

	try {
		if (newFiles.value.length > 0) {
			const formData = new FormData()
			const fileParts = newFiles.value.map((f, idx) => `${f.name}-${idx}`)

			formData.append(
				'data',
				JSON.stringify({
					file_types: newFileTypes.value.reduce(
						(acc, x, i) => ({
							...acc,
							[fileParts[i]]: x ? x.value : null,
						}),
						{},
					),
				}),
			)

			for (let i = 0; i < newFiles.value.length; i++) {
				formData.append(fileParts[i], new Blob([newFiles.value[i]]), newFiles.value[i].name)
			}

			await useBaseFetch(`version/${version.value.id}/file`, {
				method: 'POST',
				body: formData,
				headers: {
					'Content-Disposition': formData as any,
				},
			})
		}

		const body: Record<string, any> = {
			name: version.value.name || version.value.version_number,
			version_number: version.value.version_number,
			changelog: version.value.changelog,
			version_type: version.value.version_type,
			dependencies: version.value.dependencies,
			game_versions: version.value.game_versions,
			loaders: version.value.loaders,
			primary_file: ['sha1', primaryFile.value.hashes.sha1],
			featured: version.value.featured,
			file_types: oldFileTypes.value.map((x, i) => {
				return {
					algorithm: 'sha1',
					hash: version.value.files[i].hashes.sha1,
					file_type: x ? x.value : null,
				}
			}),
		}

		if (project.value.project_type === 'modpack') {
			delete body.dependencies
		}

		await useBaseFetch(`version/${version.value.id}`, {
			method: 'PATCH',
			body,
		})

		for (const hash of deleteFiles.value) {
			await useBaseFetch(`version_file/${hash}?version_id=${version.value.id}`, {
				method: 'DELETE',
			})
		}

		await resetProjectVersions()

		await router.replace(
			`/${project.value.project_type}/${project.value.slug ? project.value.slug : project.value.id}/version/${encodeURI(
				((contextVersions.value ?? []) as any[]).find((x: any) => x.id === version.value.id)
					?.displayUrlEnding ?? version.value.id,
			)}`,
		)
	} catch (err: any) {
		addNotification({
			title: 'An error occurred',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
		window.scrollTo({ top: 0, behavior: 'smooth' })
	}
	stopLoading()
}

async function createVersion() {
	shouldPreventActions.value = true
	startLoading()

	if (fieldErrors.value) {
		showKnownErrors.value = true
		shouldPreventActions.value = false
		stopLoading()
		return
	}

	try {
		await createVersionRaw(version.value)
	} catch (err: any) {
		addNotification({
			title: 'An error occurred',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
		window.scrollTo({ top: 0, behavior: 'smooth' })
	}

	stopLoading()
	shouldPreventActions.value = false
}

async function createVersionRaw(versionData: Record<string, any>) {
	const formData = new FormData()

	const fileParts = newFiles.value.map((f, idx) => `${f.name}-${idx}`)
	if (replaceFile.value) {
		fileParts.unshift(replaceFile.value.name.concat('-primary'))
	}

	if (project.value.project_type === 'resourcepack') {
		versionData.loaders = ['minecraft']
	}

	const newVersion = {
		project_id: versionData.project_id,
		file_parts: fileParts,
		version_number: versionData.version_number,
		version_title: versionData.name || versionData.version_number,
		version_body: versionData.changelog,
		dependencies: versionData.dependencies,
		game_versions: versionData.game_versions,
		loaders: versionData.loaders,
		release_channel: versionData.version_type,
		featured: versionData.featured,
		file_types: newFileTypes.value.reduce(
			(acc, x, i) => ({
				...acc,
				[fileParts[replaceFile.value ? i + 1 : i]]: x ? x.value : null,
			}),
			{},
		),
	}

	formData.append('data', JSON.stringify(newVersion))

	if (replaceFile.value) {
		formData.append(
			replaceFile.value.name.concat('-primary'),
			new Blob([replaceFile.value]),
			replaceFile.value.name,
		)
	}

	for (let i = 0; i < newFiles.value.length; i++) {
		formData.append(
			fileParts[replaceFile.value ? i + 1 : i],
			new Blob([newFiles.value[i]]),
			newFiles.value[i].name,
		)
	}

	const responseData = (await useBaseFetch('version', {
		method: 'POST',
		body: formData,
		headers: {
			'Content-Disposition': formData as any,
		},
	})) as any

	await resetProjectVersions()

	await router.push(
		`/${project.value.project_type}/${project.value.slug ? project.value.slug : project.value.id}/version/${responseData.id}`,
	)
}

async function deleteVersion() {
	startLoading()

	await useBaseFetch(`version/${version.value.id}`, {
		method: 'DELETE',
	})

	await resetProjectVersions()
	await router.replace(`/${project.value.project_type}/${project.value.id}/versions`)
	stopLoading()
}

async function createDataPackVersionHandler() {
	shouldPreventActions.value = true
	startLoading()
	try {
		const blob = await createDataPackVersion(
			project.value,
			version.value,
			primaryFile.value,
			members.value ?? [],
			tags.value.gameVersions,
			packageLoaders.value,
		)

		newFiles.value = []
		newFileTypes.value = []
		replaceFile.value = new File(
			[blob],
			`${project.value.slug}-${version.value.version_number}.jar`,
		)

		await createVersionRaw({
			project_id: project.value.id,
			author_id: currentMember.value?.user.id,
			name: version.value.name,
			version_number: `${version.value.version_number}+mod`,
			changelog: version.value.changelog,
			version_type: version.value.version_type,
			dependencies: version.value.dependencies,
			game_versions: version.value.game_versions,
			loaders: packageLoaders.value,
			featured: version.value.featured,
		})

		modal_package_mod.value?.hide()

		addNotification({
			title: 'Packaging Success',
			text: 'Your data pack was successfully packaged as a mod! Make sure to playtest to check for errors.',
			type: 'success',
		})
	} catch (err: any) {
		addNotification({
			title: 'An error occurred',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
	shouldPreventActions.value = false
}

async function resetProjectVersions() {
	await Promise.all([refreshVersions(), refreshProject()])
}
</script>

<style lang="scss" scoped>
.changelog-editor-spacing {
	padding-block: var(--gap-md);
}

.version-page {
	display: grid;

	grid-template:
		'title' auto
		'changelog' auto
		'dependencies' auto
		'metadata' auto
		'files' auto
		/ 1fr;

	@media (min-width: 1200px) {
		grid-template:
			'title title' auto
			'changelog metadata' auto
			'dependencies metadata' auto
			'files metadata' auto
			'dummy metadata' 1fr
			/ 1fr 20rem;
	}

	column-gap: var(--spacing-card-md);

	.version-page__title {
		grid-area: title;

		.version-header {
			display: flex;
			flex-wrap: wrap;
			align-items: center;
			gap: var(--spacing-card-md);

			h2,
			input[type='text'] {
				margin: 0;
				font-size: var(--font-size-2xl);
				font-weight: bold;
			}

			input[type='text'] {
				max-width: 100%;
				min-width: 0;
				flex-grow: 1;
				width: 2rem;
			}

			.featured {
				display: flex;
				align-items: center;
				gap: var(--spacing-card-xs);

				svg {
					height: 1.45rem;
				}
			}
		}

		.known-errors {
			margin-bottom: 1rem;
		}
	}

	h3 {
		font-size: var(--font-size-lg);
		margin: 0 0 0.5rem 0;
	}

	.version-page__changelog {
		grid-area: changelog;
		overflow-x: hidden;
	}

	.version-page__dependencies {
		grid-area: dependencies;

		.dependency {
			align-items: center;
			display: flex;
			gap: var(--spacing-card-sm);
			padding: var(--spacing-card-sm);

			.info {
				display: flex;
				flex-direction: column;
				gap: var(--spacing-card-xs);

				.project-title {
					font-weight: bold;
				}

				.dep-type {
					color: var(--color-text-secondary);

					&.incompatible {
						color: var(--color-red);
					}

					&::first-letter {
						text-transform: capitalize;
					}
				}
			}

			button {
				margin-left: auto;
			}
		}

		.add-dependency {
			h4 {
				margin-bottom: var(--spacing-card-sm);
			}

			.input-group {
				&:not(:last-child) {
					margin-bottom: var(--spacing-card-sm);
				}

				.multiselect {
					width: 8rem;
					flex-grow: 1;
				}

				input {
					flex-grow: 2;
				}
			}
		}
	}

	.version-page__files {
		grid-area: files;

		.file {
			--text-color: var(--color-button-text);
			--background-color: var(--color-button-bg);

			&.primary {
				--background-color: var(--color-brand-highlight);
				--text-color: var(--color-button-text-active);
			}

			display: flex;
			align-items: center;

			font-weight: 500;
			color: var(--text-color);
			background-color: var(--background-color);
			padding: var(--spacing-card-sm) var(--spacing-card-bg);
			border-radius: var(--size-rounded-sm);

			svg {
				min-width: 1.1rem;
				min-height: 1.1rem;
				margin-right: 0.5rem;
			}

			.filename {
				word-wrap: anywhere;
			}

			.file-size {
				margin-left: 1ch;
				font-weight: 400;
				white-space: nowrap;
			}

			.file-type {
				margin-left: 1ch;
				font-style: italic;
				font-weight: 300;
			}

			.raised-multiselect {
				display: none;
				margin: 0 0.5rem;
				height: 40px;
				max-height: 40px;
				min-width: 235px;
			}

			.raised-button {
				margin-left: auto;
				background-color: var(--color-raised-bg);
			}

			&:not(:nth-child(2)) {
				margin-top: 0.5rem;
			}

			// TODO: Make file type editing  work on mobile
			@media (min-width: 600px) {
				.raised-multiselect {
					display: block;
				}
			}
		}

		.additional-files {
			h4 {
				margin-bottom: 0.5rem;
			}

			label {
				margin-top: 0.5rem;
			}
		}
	}
}

.version-page__metadata {
	grid-area: metadata;

	h4 {
		margin: 1rem 0 0.25rem 0;
	}

	.maven-section {
		display: flex;
		align-items: center;
		gap: 0.5rem;

		button {
			max-width: 100%;
		}
	}

	.team-member {
		align-items: center;
		padding: 0.25rem 0.5rem;

		.member-info {
			overflow: hidden;
			margin: auto 0 auto 0.75rem;

			.name {
				font-weight: bold;
			}

			p {
				font-size: var(--font-size-sm);
				margin: 0.2rem 0;
			}
		}
	}
}

.separator {
	margin: var(--spacing-card-sm) 0;
}

.modal-package-mod {
	padding: var(--spacing-card-bg);
	display: flex;
	flex-direction: column;

	.markdown-body {
		margin-bottom: 1rem;
	}

	.multiselect {
		max-width: 20rem;
	}
}
</style>
