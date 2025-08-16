<template>
	<div v-if="version" class="version-page">
		<ConfirmModal
			v-if="currentMember"
			ref="modal_confirm"
			title="Are you sure you want to delete this version?"
			description="This will remove this version forever (like really forever)."
			:has-to-type="false"
			proceed-label="Delete"
			@proceed="deleteVersion()"
		/>
		<Modal v-if="auth.user && currentMember" ref="modalPackageMod" header="Package data pack">
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
				<Multiselect
					id="package-mod-loaders"
					v-model="packageLoaders"
					:options="['fabric', 'forge', 'quilt', 'neoforge']"
					:custom-label="(value) => value.charAt(0).toUpperCase() + value.slice(1)"
					:multiple="true"
					:searchable="false"
					:show-no-results="false"
					:show-labels="false"
					placeholder="Choose loaders..."
					open-direction="top"
				/>
				<div class="button-group">
					<ButtonStyled>
						<button @click="modalPackageMod?.hide()">
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
				<div v-if="version.featured" class="featured">
					<StarIcon aria-hidden="true" />
					Featured
				</div>
				<div v-else-if="featuredVersions.find((x) => x.id === version.id)" class="featured">
					<StarIcon aria-hidden="true" />
					Auto-featured
				</div>
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
				<ButtonStyled>
					<button @click="version.featured = !version.featured">
						<StarIcon aria-hidden="true" />
						<template v-if="!version.featured"> Feature version</template>
						<template v-else> Unfeature version</template>
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
			<div v-else class="input-group">
				<ButtonStyled v-if="primaryFile" color="brand">
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
					<button @click="reportVersion(version.id)">
						<ReportIcon aria-hidden="true" />
						Report
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<nuxt-link
						v-if="currentMember"
						class="action"
						:to="`/${project.project_type}/${
							project.slug ? project.slug : project.id
						}/version/${encodeURI(version.displayUrlEnding)}/edit`"
					>
						<EditIcon aria-hidden="true" />
						Edit
					</nuxt-link>
				</ButtonStyled>
				<ButtonStyled>
					<button
						v-if="
							currentMember &&
							version.loaders.some((x) => tags.loaderData.dataPackLoaders.includes(x))
						"
						@click="modalPackageMod?.show()"
					>
						<BoxIcon aria-hidden="true" />
						Package as mod
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<button v-if="currentMember" @click="$refs.modal_confirm.show()">
						<TrashIcon aria-hidden="true" />
						Delete
					</button>
				</ButtonStyled>
			</div>
		</div>
		<div class="version-page__changelog universal-card">
			<h3>Changelog</h3>
			<template v-if="isEditing">
				<div class="changelog-editor-spacing">
					<MarkdownEditor v-model="version.changelog" :on-image-upload="onImageUpload" />
				</div>
			</template>
			<div
				v-else
				class="markdown-body"
				v-html="
					version.changelog ? renderHighlightedString(version.changelog) : 'No changelog specified.'
				"
			/>
		</div>
		<div
			v-if="deps.length > 0 || (isEditing && project.project_type !== 'modpack')"
			class="version-page__dependencies universal-card"
		>
			<h3>Dependencies</h3>
			<div
				v-for="(dependency, index) in deps.filter((x) => !x.file_name)"
				:key="index"
				class="dependency"
				:class="{ 'button-transparent': !isEditing }"
				@click="!isEditing ? $router.push(dependency.link) : {}"
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
				v-for="(dependency, index) in deps.filter((x) => x.file_name)"
				:key="index"
				class="dependency"
			>
				<Avatar :src="null" alt="dependency-icon" size="sm" />
				<div class="info">
					<span class="project-title">
						{{ dependency.file_name }}
					</span>
					<span class="dep-type" :class="dependency.dependency_type">Added via overrides</span>
				</div>
			</div>
			<div v-if="isEditing && project.project_type !== 'modpack'" class="add-dependency">
				<h4>Add dependency</h4>
				<div class="input-group">
					<Multiselect
						v-model="dependencyAddMode"
						class="input"
						:options="['project', 'version']"
						:custom-label="(value) => value.charAt(0).toUpperCase() + value.slice(1)"
						:searchable="false"
						:close-on-select="true"
						:show-labels="false"
						:allow-empty="false"
					/>
					<input
						v-model="newDependencyId"
						type="text"
						:placeholder="`Enter the ${dependencyAddMode} ID${
							dependencyAddMode === 'project' ? '/slug' : ''
						}`"
						@keyup.enter="addDependency(dependencyAddMode, newDependencyId, newDependencyType)"
					/>
					<Multiselect
						v-model="newDependencyType"
						class="input"
						:options="['required', 'optional', 'incompatible', 'embedded']"
						:custom-label="(value) => value.charAt(0).toUpperCase() + value.slice(1)"
						:searchable="false"
						:close-on-select="true"
						:show-labels="false"
						:allow-empty="true"
					/>
				</div>
				<div class="input-group">
					<ButtonStyled color="brand">
						<button @click="addDependency(dependencyAddMode, newDependencyId, newDependencyType)">
							<PlusIcon aria-hidden="true" />
							Add dependency
						</button>
					</ButtonStyled>
				</div>
			</div>
		</div>
		<div class="version-page__files universal-card">
			<h3>Files</h3>
			<div v-if="isEditing && replaceFile" class="file primary">
				<FileIcon aria-hidden="true" />
				<span class="filename">
					<strong>{{ replaceFile.name }}</strong>
					<span class="file-size">({{ formatBytes(replaceFile.size) }})</span>
				</span>
				<FileInput
					class="iconified-button raised-button"
					prompt="Replace"
					aria-label="Replace"
					:accept="acceptFileFromProjectType(project.project_type)"
					:max-size="524288000"
					should-always-reset
					@change="(x) => (replaceFile = x[0])"
				>
					<TransferIcon aria-hidden="true" />
				</FileInput>
			</div>
			<div
				v-for="(file, index) in version.files"
				:key="file.hashes.sha1"
				:class="{
					file: true,
					primary: primaryFile?.hashes?.sha1 === file.hashes.sha1,
				}"
			>
				<FileIcon aria-hidden="true" />
				<span class="filename">
					<strong>{{ file.filename }}</strong>
					<span class="file-size">({{ formatBytes(file.size) }})</span>
					<span v-if="primaryFile?.hashes?.sha1 === file.hashes.sha1" class="file-type">
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
				<Multiselect
					v-if="
						version.loaders.some((x) => tags.loaderData.dataPackLoaders.includes(x)) &&
						isEditing &&
						primaryFile?.hashes?.sha1 !== file.hashes.sha1
					"
					v-model="oldFileTypes[index]"
					class="raised-multiselect"
					placeholder="Select file type"
					:options="fileTypes"
					track-by="value"
					label="display"
					:searchable="false"
					:close-on-select="true"
					:show-labels="false"
					:allow-empty="false"
				/>
				<ButtonStyled v-if="isEditing">
					<button
						class="raised-button"
						:disabled="primaryFile?.hashes?.sha1 === file.hashes.sha1"
						@click="
							() => {
								deleteFiles.push(file.hashes.sha1)
								version.files.splice(index, 1)
								oldFileTypes.splice(index, 1)
							}
						"
					>
						<TrashIcon aria-hidden="true" />
						Remove
					</button>
				</ButtonStyled>
				<ButtonStyled v-else>
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
			<template v-if="isEditing">
				<div v-for="(file, index) in newFiles" :key="index" class="file">
					<FileIcon aria-hidden="true" />
					<span class="filename">
						<strong>{{ file.name }}</strong>
						<span class="file-size">({{ formatBytes(file.size) }})</span>
					</span>
					<Multiselect
						v-if="version.loaders.some((x) => tags.loaderData.dataPackLoaders.includes(x))"
						v-model="newFileTypes[index]"
						class="raised-multiselect"
						placeholder="Select file type"
						:options="fileTypes"
						track-by="value"
						label="display"
						:searchable="false"
						:close-on-select="true"
						:show-labels="false"
						:allow-empty="false"
					/>
					<ButtonStyled>
						<button
							class="raised-button"
							@click="
								() => {
									newFiles.splice(index, 1)
									newFileTypes.splice(index, 1)
								}
							"
						>
							<TrashIcon aria-hidden="true" />
							Remove
						</button>
					</ButtonStyled>
				</div>
				<div class="additional-files">
					<h4>Upload additional files</h4>
					<span v-if="version.loaders.some((x) => tags.loaderData.dataPackLoaders.includes(x))">
						Used for additional files such as required/optional resource packs
					</span>
					<span v-else>Used for files such as sources or Javadocs.</span>
					<FileInput
						prompt="Drag and drop to upload or click to select"
						aria-label="Upload additional file"
						multiple
						long-style
						:accept="acceptFileFromProjectType(project.project_type)"
						:max-size="524288000"
						@change="
							(x) =>
								x.forEach((y) => {
									newFiles.push(y)
									newFileTypes.push(null)
								})
						"
					>
						<UploadIcon aria-hidden="true" />
					</FileInput>
				</div>
			</template>
		</div>
		<div class="version-page__metadata">
			<div class="universal-card full-width-inputs">
				<h3>Metadata</h3>
				<div>
					<h4>Release channel</h4>
					<Multiselect
						v-if="isEditing"
						v-model="version.version_type"
						class="input"
						placeholder="Select one"
						:options="['release', 'beta', 'alpha']"
						:custom-label="(value) => value.charAt(0).toUpperCase() + value.slice(1)"
						:searchable="false"
						:close-on-select="true"
						:show-labels="false"
						:allow-empty="false"
					/>
					<template v-else>
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
					</template>
				</div>
				<div>
					<h4>Version number</h4>
					<div v-if="isEditing" class="iconified-input">
						<label class="hidden" for="version-number">Version number</label>
						<HashIcon aria-hidden="true" />
						<input
							id="version-number"
							v-model="version.version_number"
							type="text"
							autocomplete="off"
							maxlength="32"
						/>
					</div>
					<span v-else>{{ version.version_number }}</span>
				</div>
				<div v-if="project.project_type !== 'resourcepack'">
					<h4>Loaders</h4>
					<Multiselect
						v-if="isEditing"
						v-model="version.loaders"
						:options="
							tags.loaders
								.filter((x) =>
									x.supported_project_types.includes(project.actualProjectType.toLowerCase()),
								)
								.map((it) => it.name)
						"
						:custom-label="formatCategory"
						:loading="tags.loaders.length === 0"
						:multiple="true"
						:searchable="true"
						:show-no-results="false"
						:close-on-select="false"
						:clear-on-select="false"
						:show-labels="false"
						:hide-selected="true"
						placeholder="Choose loaders..."
					/>
					<Categories v-else :categories="version.loaders" :type="project.actualProjectType" />
				</div>
				<div>
					<h4>Game versions</h4>
					<template v-if="isEditing">
						<Multiselect
							v-model="version.game_versions"
							:options="
								showSnapshots
									? tags.gameVersions.map((x) => x.version)
									: tags.gameVersions
											.filter((it) => it.version_type === 'release')
											.map((x) => x.version)
							"
							:loading="tags.gameVersions.length === 0"
							:multiple="true"
							:searchable="true"
							:show-no-results="false"
							:close-on-select="false"
							:clear-on-select="false"
							:show-labels="false"
							:hide-selected="true"
							:custom-label="(version) => version"
							placeholder="Choose versions..."
						/>
						<Checkbox
							v-model="showSnapshots"
							label="Show all versions"
							description="Show all versions"
							style="margin-top: 0.5rem"
							:border="false"
						/>
					</template>
					<span v-else>{{ $formatVersion(version.game_versions) }}</span>
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
						@click="$router.push('/user/' + version.author.user.username)"
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

<script setup>
import {
	BoxIcon,
	DownloadIcon,
	EditIcon,
	FileIcon,
	HashIcon,
	PlusIcon,
	ReportIcon,
	RightArrowIcon,
	SaveIcon,
	StarIcon,
	TransferIcon,
	TrashIcon,
	UploadIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Badge,
	ButtonStyled,
	Categories,
	Checkbox,
	ConfirmModal,
	CopyCode,
	injectNotificationManager,
	MarkdownEditor,
} from '@modrinth/ui'
import { formatBytes, formatCategory } from '@modrinth/utils'
import { Multiselect } from 'vue-multiselect'

import Breadcrumbs from '~/components/ui/Breadcrumbs.vue'
import FileInput from '~/components/ui/FileInput.vue'
import Modal from '~/components/ui/Modal.vue'
import { useImageUpload } from '~/composables/image-upload.ts'
import { acceptFileFromProjectType } from '~/helpers/fileUtils.js'
import { renderHighlightedString } from '~/helpers/highlight.js'
import { inferVersionInfo } from '~/helpers/infer.js'
import { createDataPackVersion } from '~/helpers/package.js'
import { reportVersion } from '~/utils/report-helpers.ts'

const props = defineProps({
	project: { type: Object, default: () => ({}) },
	versions: { type: Array, default: () => [] },
	featuredVersions: { type: Array, default: () => [] },
	members: { type: Array, default: () => [{}] },
	currentMember: { type: Object, default: () => null },
	dependencies: { type: Object, default: () => ({}) },
	resetProject: {
		type: Function,
		required: true,
		default: () => {},
	},
})

const emit = defineEmits([
	'update:dependencies',
	'update:versions',
	'update:featuredVersions',
	'onDownload',
])

const nuxtApp = useNuxtApp()
const router = useRouter()
const route = typeof useNativeRoute === 'function' ? useNativeRoute() : useRoute()

const auth = await useAuth()
const tags = useTags()
const flags = useFeatureFlags()

const path = String(route.name || '').split('-')
const mode = path[path.length - 1]

const fileTypesConst = [
	{ display: 'Required resource pack', value: 'required-resource-pack' },
	{ display: 'Optional resource pack', value: 'optional-resource-pack' },
]

const fileTypes = ref(fileTypesConst)
const oldFileTypes = ref([])

const isCreating = ref(false)
const isEditing = ref(false)

const version = ref({})
const primaryFile = ref({})
const alternateFile = ref({})
const replaceFile = ref(null)
const uploadedImageIds = ref([])

const dependencyAddMode = ref('project')
const newDependencyType = ref('required')
const newDependencyId = ref('')

const showSnapshots = ref(false)

const newFiles = ref([])
const deleteFiles = ref([])
const newFileTypes = ref([])

const packageLoaders = ref(['forge', 'fabric', 'quilt', 'neoforge'])

const showKnownErrors = ref(false)
const shouldPreventActions = ref(false)

const { addNotification } = injectNotificationManager()

{
	if (mode === 'edit') {
		isEditing.value = true
	}

	if (route.params.version === 'create') {
		isCreating.value = true
		isEditing.value = true

		version.value = {
			id: 'none',
			project_id: props.project.id,
			author_id: props.currentMember?.user?.id,
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

		if (import.meta.client && history.state && history.state.newPrimaryFile) {
			replaceFile.value = history.state.newPrimaryFile
			try {
				const inferredData = await inferVersionInfo(
					replaceFile.value,
					props.project,
					tags.value.gameVersions,
				)
				version.value = { ...version.value, ...inferredData }
			} catch (err) {
				console.error('Error parsing version file data', err)
			}
		}
	} else if (route.params.version === 'latest') {
		let versionList = props.versions
		if (route.query.loader) {
			versionList = versionList.filter((x) => x.loaders.includes(route.query.loader))
		}
		if (route.query.version) {
			versionList = versionList.filter((x) => x.game_versions.includes(route.query.version))
		}
		if (versionList.length === 0) {
			throw createError({
				fatal: true,
				statusCode: 404,
				message: 'No version matches the filters',
			})
		}
		version.value = versionList.reduce((a, b) => (a.date_published > b.date_published ? a : b))
	} else {
		version.value = props.versions.find((x) => x.id === route.params.version)
		if (!version.value) {
			version.value = props.versions.find((x) => x.displayUrlEnding === route.params.version)
		}
	}

	if (!version.value) {
		throw createError({ fatal: true, statusCode: 404, message: 'Version not found' })
	}

	version.value = JSON.parse(JSON.stringify(version.value))
	primaryFile.value = version.value.files.find((file) => file.primary) ?? version.value.files[0]
	alternateFile.value = version.value.files.find(
		(file) => file.file_type && file.file_type.includes('resource-pack'),
	)

	for (const dependency of version.value.dependencies) {
		dependency.version = props.dependencies.versions.find((x) => x.id === dependency.version_id)
		if (dependency.version) {
			dependency.project = props.dependencies.projects.find(
				(x) => x.id === dependency.version.project_id,
			)
		}
		if (!dependency.project) {
			dependency.project = props.dependencies.projects.find((x) => x.id === dependency.project_id)
		}
		dependency.link = dependency.project
			? `/${dependency.project.project_type}/${dependency.project.slug ?? dependency.project.id}${
					dependency.version ? `/version/${encodeURI(dependency.version.version_number)}` : ''
				}`
			: ''
	}

	oldFileTypes.value = version.value.files.map((x) =>
		fileTypesConst.find((y) => y.value === x.file_type),
	)
}

const title = computed(
	() => `${isCreating.value ? 'Create Version' : version.value.name} - ${props.project.title}`,
)
const description = computed(
	() =>
		`Download ${props.project.title} ${version.value.version_number} on Modrinth. Supports ${nuxtApp.$formatVersion(
			version.value.game_versions,
		)} ${version.value.loaders
			.map((x) => x.charAt(0).toUpperCase() + x.slice(1))
			.join(
				' & ',
			)}. Published on ${nuxtApp.$dayjs(version.value.date_published).format('MMM D, YYYY')}. ${
			version.value.downloads
		} downloads.`,
)

useSeoMeta({
	title,
	description,
	ogTitle: title,
	ogDescription: description,
})

const fieldErrors = computed(
	() =>
		version.value.version_number === '' ||
		version.value.game_versions.length === 0 ||
		(version.value.loaders.length === 0 && props.project.project_type !== 'resourcepack') ||
		(newFiles.value.length === 0 && version.value.files.length === 0 && !replaceFile.value),
)

const deps = computed(() => {
	const order = ['required', 'optional', 'incompatible', 'embedded']
	return [...version.value.dependencies].sort(
		(a, b) => order.indexOf(a.dependency_type) - order.indexOf(b.dependency_type),
	)
})

watch(
	() => route.path,
	() => {
		const p = String(route.name || '').split('-')
		const m = p[p.length - 1]
		isEditing.value = m === 'edit' || route.params.version === 'create'
	},
)

const onImageUpload = async (file) => {
	const response = await useImageUpload(file, { context: 'version' })
	uploadedImageIds.value.push(response.id)
	uploadedImageIds.value = uploadedImageIds.value.slice(-10)
	return response.url
}

const getPreviousLink = () => {
	const back = router.options?.history?.state?.back
	if (back && back.includes('/versions')) return back
	return `/${props.project.project_type}/${props.project.slug ? props.project.slug : props.project.id}/versions`
}

const getPreviousLabel = () => {
	const back = router.options?.history?.state?.back
	return back && back.endsWith('/versions') ? 'Back to versions' : 'All versions'
}

const addDependency = async (modeArg, idArg, typeArg, hideErrors) => {
	try {
		if (modeArg === 'project') {
			const project = await useBaseFetch(`project/${idArg}`)
			if (version.value.dependencies.some((dep) => project.id === dep.project_id)) {
				addNotification({
					title: 'Dependency already added',
					text: 'You cannot add the same dependency twice.',
					type: 'error',
				})
			} else {
				version.value.dependencies.push({
					project,
					project_id: project.id,
					dependency_type: typeArg,
					link: `/${project.project_type}/${project.slug ?? project.id}`,
				})
				emit('update:dependencies', {
					projects: props.dependencies.projects.concat([project]),
					versions: props.dependencies.versions,
				})
			}
		} else if (modeArg === 'version') {
			const v = await useBaseFetch(`version/${idArg}`)
			const project = await useBaseFetch(`project/${v.project_id}`)

			if (version.value.dependencies.some((dep) => v.id === dep.version_id)) {
				addNotification({
					title: 'Dependency already added',
					text: 'You cannot add the same dependency twice.',
					type: 'error',
				})
			} else {
				version.value.dependencies.push({
					version: v,
					project,
					version_id: v.id,
					project_id: project.id,
					dependency_type: typeArg,
					link: `/${project.project_type}/${project.slug ?? project.id}/version/${encodeURI(v.version_number)}`,
				})
				emit('update:dependencies', {
					projects: props.dependencies.projects.concat([project]),
					versions: props.dependencies.versions.concat([v]),
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

const loadingIndicator = typeof useLoadingIndicator === 'function' ? useLoadingIndicator() : null
const startLoading = () => {
	try {
		loadingIndicator?.start?.()
	} catch {
		// Ignored
	}
}
const stopLoading = () => {
	try {
		loadingIndicator?.finish?.()
	} catch {
		// Ignored
	}
}

const saveEditedVersion = async () => {
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
				headers: { 'Content-Disposition': formData },
			})
		}

		const body = {
			name: version.value.name || version.value.version_number,
			version_number: version.value.version_number,
			changelog: version.value.changelog,
			version_type: version.value.version_type,
			dependencies: version.value.dependencies,
			game_versions: version.value.game_versions,
			loaders: version.value.loaders,
			primary_file: ['sha1', primaryFile.value.hashes.sha1],
			featured: version.value.featured,
			file_types: oldFileTypes.value.map((x, i) => ({
				algorithm: 'sha1',
				hash: version.value.files[i].hashes.sha1,
				file_type: x ? x.value : null,
			})),
		}

		if (props.project.project_type === 'modpack') {
			delete body.dependencies
		}

		await useBaseFetch(`version/${version.value.id}`, { method: 'PATCH', body })

		for (const hash of deleteFiles.value) {
			await useBaseFetch(`version_file/${hash}?version_id=${version.value.id}`, {
				method: 'DELETE',
			})
		}

		const created = await resetProjectVersions()

		await router.replace(
			`/${props.project.project_type}/${
				props.project.slug ? props.project.slug : props.project.id
			}/version/${encodeURI(created.find((x) => x.id === version.value.id).displayUrlEnding)}`,
		)
	} catch (err) {
		addNotification({
			title: 'An error occurred',
			text: err?.data ? err.data.description : err,
			type: 'error',
		})
		window.scrollTo({ top: 0, behavior: 'smooth' })
	}
	stopLoading()
}

const createVersion = async () => {
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
	} catch (err) {
		addNotification({
			title: 'An error occurred',
			text: err?.data ? err.data.description : err,
			type: 'error',
		})
		window.scrollTo({ top: 0, behavior: 'smooth' })
	}

	stopLoading()
	shouldPreventActions.value = false
}

const createVersionRaw = async (ver) => {
	const formData = new FormData()
	const fileParts = newFiles.value.map((f, idx) => `${f.name}-${idx}`)
	if (replaceFile.value) {
		fileParts.unshift(replaceFile.value.name.concat('-primary'))
	}

	if (props.project.project_type === 'resourcepack') {
		ver.loaders = ['minecraft']
	}

	const newVersion = {
		project_id: ver.project_id,
		file_parts: fileParts,
		version_number: ver.version_number,
		version_title: ver.name || ver.version_number,
		version_body: ver.changelog,
		dependencies: ver.dependencies,
		game_versions: ver.game_versions,
		loaders: ver.loaders,
		release_channel: ver.version_type,
		featured: ver.featured,
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

	const data = await useBaseFetch('version', {
		method: 'POST',
		body: formData,
		headers: { 'Content-Disposition': formData },
	})

	await resetProjectVersions()

	await router.push(
		`/${props.project.project_type}/${
			props.project.slug ? props.project.slug : props.project.id
		}/version/${data.id}`,
	)
}

const deleteVersion = async () => {
	startLoading()
	await useBaseFetch(`version/${version.value.id}`, { method: 'DELETE' })
	await resetProjectVersions()
	await router.replace(`/${props.project.project_type}/${props.project.id}/versions`)
	stopLoading()
}

const modalPackageMod = ref(null)

const createDataPackVersionHandler = async () => {
	shouldPreventActions.value = true
	startLoading()
	try {
		const blob = await createDataPackVersion(
			props.project,
			version.value,
			primaryFile.value,
			props.members,
			tags.value.gameVersions,
			packageLoaders.value,
		)

		newFiles.value = []
		newFileTypes.value = []
		replaceFile.value = new File(
			[blob],
			`${props.project.slug}-${version.value.version_number}.jar`,
		)

		await createVersionRaw({
			project_id: props.project.id,
			author_id: props.currentMember.user.id,
			name: version.value.name,
			version_number: `${version.value.version_number}+mod`,
			changelog: version.value.changelog,
			version_type: version.value.version_type,
			dependencies: version.value.dependencies,
			game_versions: version.value.game_versions,
			loaders: packageLoaders.value,
			featured: version.value.featured,
		})

		modalPackageMod.value?.hide()

		addNotification({
			title: 'Packaging Success',
			text: 'Your data pack was successfully packaged as a mod! Make sure to playtest to check for errors.',
			type: 'success',
		})
	} catch (err) {
		addNotification({
			title: 'An error occurred',
			text: err?.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
	shouldPreventActions.value = false
}

const resetProjectVersions = async () => {
	const [versionsRes, featuredVersionsRes, dependenciesRes] = await Promise.all([
		useBaseFetch(`project/${version.value.project_id}/version`),
		useBaseFetch(`project/${version.value.project_id}/version?featured=true`),
		useBaseFetch(`project/${version.value.project_id}/dependencies`),
		props.resetProject(),
	])

	const newCreatedVersions = nuxtApp.$computeVersions(versionsRes, props.members)
	const featuredIds = featuredVersionsRes.map((x) => x.id)

	emit('update:versions', newCreatedVersions)
	emit(
		'update:featuredVersions',
		newCreatedVersions.filter((v) => featuredIds.includes(v.id)),
	)
	emit('update:dependencies', dependenciesRes)

	return newCreatedVersions
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
			margin-bottom: 1rem;
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
