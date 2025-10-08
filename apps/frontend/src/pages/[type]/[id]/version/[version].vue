<!-- eslint-disable vue/no-undef-components -->
<!-- TODO: Remove this^after converting to composition API. -->
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
						<button @click="$refs.modal_package_mod.hide()">
							<XIcon aria-hidden="true" />
							Cancel
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button @click="createDataPackVersion">
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
					<button @click="() => reportVersion(version.id)">
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
						@click="$refs.modal_package_mod.show()"
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
				<div class="dependency-filters">
					<Multiselect
						v-model="dependencyFilterType"
						class="filter-select"
						:options="['all', 'mod', 'plugin', 'resourcepack', 'datapack', 'shader']"
						:custom-label="(value) => value === 'all' ? 'All Types' : value.charAt(0).toUpperCase() + value.slice(1)"
						:searchable="false"
						:close-on-select="true"
						:show-labels="false"
						:allow-empty="false"
						@update:model-value="onFilterChange"
					/>
					<Multiselect
						v-model="dependencyFilterCompatibility"
						class="filter-select"
						:options="['all', 'compatible', 'unknown', 'incompatible']"
						:custom-label="(value) => value === 'all' ? 'All Compatibility' : value.charAt(0).toUpperCase() + value.slice(1)"
						:searchable="false"
						:close-on-select="true"
						:show-labels="false"
						:allow-empty="false"
						@update:model-value="onFilterChange"
					/>
				</div>
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
					<Multiselect
						v-if="dependencyAddMode === 'project'"
						v-model="selectedProject"
						class="input dependency-search-select"
						:options="projectSearchResults"
						:custom-label="(project) => project?.title || ''"
						track-by="project_id"
						:searchable="true"
						:internal-search="false"
						:loading="isSearchingProjects"
						:close-on-select="true"
						:show-labels="false"
						:placeholder="'Search for a project...'"
						@search-change="searchProjects"
						@select="onProjectSelect"
					>
						<template #option="{ option }">
						<div class="dependency-search-option">
							<Avatar :src="option.icon_url" alt="Project icon" size="sm" />
							<div class="dependency-search-info">
								<div class="dependency-search-title-row">
									<span class="dependency-search-title">{{ option.title }}</span>
									<span v-if="option.__matchScore" class="dependency-search-score" :title="'Relevance: ' + option.__matchScore.toFixed(2)">
										{{ Math.min(100, Math.round(option.__matchScore)).toFixed(0) }}%
									</span>
								</div>
								<span class="dependency-search-meta">
									by {{ option.author }} • {{ formatNumber(option.downloads) }} downloads
								</span>
								<div class="dependency-search-badges">
									<Badge
										v-if="computeCompatibility(option).status === 'compatible'"
										color="green"
										type="compatibility"
										:title="'Compatible with your project - Good match for loaders and game versions (Score: ' + computeCompatibility(option).score + '%)'">
										✓ Compatible
									</Badge>
									<Badge
										v-else-if="computeCompatibility(option).status === 'incompatible'"
										color="red"
										type="compatibility"
										:title="'Incompatible - May not work with your project type, loaders, or game versions'">
										✗ Incompatible
									</Badge>
									<Badge
										v-else
										color="gray"
										type="compatibility"
										:title="'Unknown compatibility - Limited version/loader overlap (Score: ' + computeCompatibility(option).score + '%)'">
										? Unknown
									</Badge>
									<Badge
										v-if="option.hasDependencies"
										color="blue"
										type="info"
										:title="'This project requires ' + (option.dependencyCount || 0) + ' other project(s) to work. Use \'Add with dependencies\' to install them automatically.'">
										{{ (option.dependencyCount || 0) + ' ' + (option.dependencyCount === 1 ? 'dependency' : 'dependencies') }}
									</Badge>
								</div>
						</div>
						</div>
					</template>
					<template #noResult>
						<span>No projects found</span>
					</template>
					<template #noOptions>
						<span>Start typing to search...</span>
						</template>
					</Multiselect>
					<input
						v-else
						v-model="newDependencyId"
						type="text"
						:placeholder="`Enter the ${dependencyAddMode} ID`"
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
					<ButtonStyled v-if="selectedProject && dependencyAddMode === 'project'">
						<button @click="addDependencyWithTransitive(newDependencyId, newDependencyType)">
							<PlusIcon aria-hidden="true" />
							Add with dependencies
						</button>
					</ButtonStyled>
				</div>
				<div v-if="transitiveDependencies.length > 0" class="transitive-dependencies">
					<h5>Dependencies to be added ({{ transitiveDependencies.length }}):</h5>
					<div class="transitive-list">
						<div v-for="dep in transitiveDependencies" :key="dep.project_id" class="transitive-item">
							<Avatar :src="dep.icon_url" alt="dependency icon" size="xs" />
							<span class="transitive-title">{{ dep.title }}</span>
							<Badge class="transitive-badge" :color="dep.dependency_type === 'required' ? 'red' : 'orange'" :type="dep.dependency_type"></Badge>
						</div>
					</div>
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
				<multiselect
					v-if="
						version.loaders.some((x) => tags.loaderData.dataPackLoaders.includes(x)) &&
						isEditing &&
						primaryFile.hashes.sha1 !== file.hashes.sha1
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
						:disabled="primaryFile.hashes.sha1 === file.hashes.sha1"
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
					<multiselect
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
						<multiselect
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
<script>
import {
	BoxIcon,
	ChevronRightIcon,
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
	Checkbox,
	ConfirmModal,
	CopyCode,
	injectNotificationManager,
	MarkdownEditor,
} from '@modrinth/ui'
import { formatBytes, formatCategory } from '@modrinth/utils'
import { Multiselect } from 'vue-multiselect'

import AdPlaceholder from '~/components/ui/AdPlaceholder.vue'
import Breadcrumbs from '~/components/ui/Breadcrumbs.vue'
import FileInput from '~/components/ui/FileInput.vue'
import Modal from '~/components/ui/Modal.vue'
import Categories from '~/components/ui/search/Categories.vue'
import { useImageUpload } from '~/composables/image-upload.ts'
import { acceptFileFromProjectType } from '~/helpers/fileUtils.js'
import { renderHighlightedString } from '~/helpers/highlight.js'
import { inferVersionInfo } from '~/helpers/infer.js'
import { createDataPackVersion } from '~/helpers/package.js'
import { reportVersion } from '~/utils/report-helpers.ts'

export default defineNuxtComponent({
	components: {
		MarkdownEditor,
		Modal,
		FileInput,
		Checkbox,
		ChevronRightIcon,
		Categories,
		DownloadIcon,
		EditIcon,
		TrashIcon,
		StarIcon,
		FileIcon,
		ReportIcon,
		SaveIcon,
		XIcon,
		HashIcon,
		PlusIcon,
		TransferIcon,
		UploadIcon,
		Avatar,
		Badge,
		Breadcrumbs,
		CopyCode,
		Multiselect,
		BoxIcon,
		RightArrowIcon,
		ConfirmModal,
		ButtonStyled,
		AdPlaceholder,
	},
	props: {
		project: {
			type: Object,
			default() {
				return {}
			},
		},
		versions: {
			type: Array,
			default() {
				return []
			},
		},
		members: {
			type: Array,
			default() {
				return [{}]
			},
		},
		currentMember: {
			type: Object,
			default() {
				return null
			},
		},
		dependencies: {
			type: Object,
			default() {
				return {}
			},
		},
		resetProject: {
			type: Function,
			required: true,
			default: () => {},
		},
	},
	async setup(props) {
		const data = useNuxtApp()
		const route = useNativeRoute()

		const auth = await useAuth()
		const tags = useTags()
		const flags = useFeatureFlags()

		const path = route.name.split('-')
		const mode = path[path.length - 1]

		const fileTypes = [
			{
				display: 'Required resource pack',
				value: 'required-resource-pack',
			},
			{
				display: 'Optional resource pack',
				value: 'optional-resource-pack',
			},
		]
		let oldFileTypes = []

		let isCreating = false
		let isEditing = false

		let version = {}
		let primaryFile = {}
		let alternateFile = {}

		let replaceFile = null

		if (mode === 'edit') {
			isEditing = true
		}

		if (route.params.version === 'create') {
			isCreating = true
			isEditing = true

			version = {
				id: 'none',
				project_id: props.project.id,
				author_id: props.currentMember.user.id,
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
				replaceFile = history.state.newPrimaryFile

				try {
					const inferredData = await inferVersionInfo(
						replaceFile,
						props.project,
						tags.value.gameVersions,
					)

					version = {
						...version,
						...inferredData,
					}
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
			version = versionList.reduce((a, b) => (a.date_published > b.date_published ? a : b))
		} else {
			version = props.versions.find((x) => x.id === route.params.version)

			if (!version) {
				version = props.versions.find((x) => x.displayUrlEnding === route.params.version)
			}
		}

		if (!version) {
			throw createError({
				fatal: true,
				statusCode: 404,
				message: 'Version not found',
			})
		}

		version = JSON.parse(JSON.stringify(version))
		primaryFile = version.files.find((file) => file.primary) ?? version.files[0]
		alternateFile = version.files.find(
			(file) => file.file_type && file.file_type.includes('resource-pack'),
		)

		for (const dependency of version.dependencies) {
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

		oldFileTypes = version.files.map((x) => fileTypes.find((y) => y.value === x.file_type))

		const title = computed(
			() => `${isCreating ? 'Create Version' : version.name} - ${props.project.title}`,
		)
		const description = computed(
			() =>
				`Download ${props.project.title} ${
					version.version_number
				} on Modrinth. Supports ${data.$formatVersion(version.game_versions)} ${version.loaders
					.map((x) => x.charAt(0).toUpperCase() + x.slice(1))
					.join(' & ')}. Published on ${data
					.$dayjs(version.date_published)
					.format('MMM D, YYYY')}. ${version.downloads} downloads.`,
		)

		const usesFeaturedVersions = computed(() => props.versions.some((v) => v.featured))

		useSeoMeta({
			title,
			description,
			ogTitle: title,
			ogDescription: description,
		})

		return {
			auth,
			tags,
			flags,
			usesFeaturedVersions,
			fileTypes: ref(fileTypes),
			oldFileTypes: ref(oldFileTypes),
			isCreating: ref(isCreating),
			isEditing: ref(isEditing),
			version: ref(version),
			primaryFile: ref(primaryFile),
			alternateFile: ref(alternateFile),
			replaceFile: ref(replaceFile),
			uploadedImageIds: ref([]),
		}
	},
	data() {
		return {
			dependencyAddMode: 'project',
			newDependencyType: 'required',
			newDependencyId: '',
			selectedProject: null,
			projectSearchResults: [],
			isSearchingProjects: false,
			searchDebounce: null,
			dependencyFilterType: 'all',
			dependencyFilterCompatibility: 'all',
			transitiveDependencies: [],
			lastSearchQuery: '',

			showSnapshots: false,

			newFiles: [],
			deleteFiles: [],
			replaceFile: null,

			newFileTypes: [],

			packageLoaders: ['forge', 'fabric', 'quilt', 'neoforge'],

			showKnownErrors: false,
			shouldPreventActions: false,
		}
	},
	created() {
		const { addNotification } = injectNotificationManager()
		this.addNotification = addNotification
	},
	computed: {
		fieldErrors() {
			return (
				this.version.version_number === '' ||
				this.version.game_versions.length === 0 ||
				(this.version.loaders.length === 0 && this.project.project_type !== 'resourcepack') ||
				(this.newFiles.length === 0 && this.version.files.length === 0 && !this.replaceFile)
			)
		},
		deps() {
			const order = ['required', 'optional', 'incompatible', 'embedded']
			return [...this.version.dependencies].sort(
				(a, b) => order.indexOf(a.dependency_type) - order.indexOf(b.dependency_type),
			)
		},
	},
	watch: {
		'$route.path'() {
			const path = this.$route.name.split('-')
			const mode = path[path.length - 1]

			this.isEditing = mode === 'edit' || this.$route.params.version === 'create'
		},
		dependencyAddMode() {
			this.selectedProject = null
			this.projectSearchResults = []
			this.newDependencyId = ''
			this.transitiveDependencies = []
		},
		selectedProject(newVal) {
			if (newVal && this.dependencyAddMode === 'project') {
				this.fetchTransitiveDependencies(newVal.project_id || newVal.slug)
			} else {
				this.transitiveDependencies = []
			}
		},
	},
	methods: {
		formatBytes,
		formatCategory,
		formatNumber(num) {
			if (num >= 1000000) {
				return (num / 1000000).toFixed(1) + 'M'
			} else if (num >= 1000) {
				return (num / 1000).toFixed(1) + 'K'
			}
			return num.toString()
		},
		levenshteinDistance(str1, str2) {
			const s1 = str1.toLowerCase()
			const s2 = str2.toLowerCase()
			const len1 = s1.length
			const len2 = s2.length
			const matrix = []

			if (len1 === 0) return len2
			if (len2 === 0) return len1

			for (let i = 0; i <= len2; i++) {
				matrix[i] = [i]
			}

			for (let j = 0; j <= len1; j++) {
				matrix[0][j] = j
			}

			for (let i = 1; i <= len2; i++) {
				for (let j = 1; j <= len1; j++) {
					if (s2.charAt(i - 1) === s1.charAt(j - 1)) {
						matrix[i][j] = matrix[i - 1][j - 1]
					} else {
						matrix[i][j] = Math.min(
							matrix[i - 1][j - 1] + 1,
							matrix[i][j - 1] + 1,
							matrix[i - 1][j] + 1,
						)
					}
				}
			}

			return matrix[len2][len1]
		},
		computeCompatibility(project) {
			try {
				let score = 0
				let maxScore = 0
				const currentType = this.project?.actualProjectType || this.project?.project_type

				// Project type compatibility (40 points)
				maxScore += 40
				if (project?.project_type && currentType) {
					if (project.project_type === currentType) {
						score += 40
					} else {
						const incompatiblePairs = new Set(['plugin:mod', 'mod:plugin'])
						const isStrictlyIncompatible = incompatiblePairs.has(`${project.project_type}:${currentType}`)
						const userExplicitlyWantsThisType =
							this.dependencyFilterType !== 'all' &&
							this.dependencyFilterType === project.project_type

						if (isStrictlyIncompatible && !userExplicitlyWantsThisType) {
							return { status: 'incompatible', score: 0, reasons: ['Incompatible project types'] }
						}

						// Partial compatibility for related types
						if (
							(project.project_type === 'resourcepack' && currentType === 'mod') ||
							(project.project_type === 'datapack' && currentType === 'mod') ||
							(project.project_type === 'shader' && currentType === 'mod')
						) {
							score += 20
						} else if (userExplicitlyWantsThisType) {
							score += 25
						} else if (isStrictlyIncompatible) {
							score += 5
						}
					}
				}

				// Game version compatibility (30 points)
				maxScore += 30
				if (project?.game_versions && this.version?.game_versions) {
					const overlap = project.game_versions.filter((v) =>
						this.version.game_versions.includes(v),
					)
					if (overlap.length > 0) {
						const overlapRatio = overlap.length / Math.max(project.game_versions.length, 1)
						const points = 30 * overlapRatio
						score += points
					}
				}

				// Loader compatibility (20 points)
				maxScore += 20
				if (project?.loaders && this.version?.loaders && this.version.loaders.length > 0) {
					const loaderOverlap = project.loaders.filter((l) => this.version.loaders.includes(l))
					if (loaderOverlap.length > 0) {
						score += 20
					} else {
						const userExplicitlyWantsThisType =
							this.dependencyFilterType !== 'all' &&
							this.dependencyFilterType === project.project_type

						const crossCompatibleTypes = ['datapack', 'plugin', 'resourcepack', 'shader']
						if (userExplicitlyWantsThisType && crossCompatibleTypes.includes(project.project_type)) {
							score += 15
						}
					}
				}

				// Environment compatibility (10 points)
				maxScore += 10
				if (project?.client_side && project?.server_side) {
					if (
						(project.client_side === 'required' || project.client_side === 'optional') &&
						(project.server_side === 'required' || project.server_side === 'optional')
					) {
						score += 10
					} else {
						score += 5
					}
				}

				const percentage = Math.round((score / maxScore) * 100)

				if (percentage >= 70) {
					return { status: 'compatible', score: percentage, reasons: [] }
				} else if (percentage >= 30) {
					return { status: 'unknown', score: percentage, reasons: [] }
				} else {
					return { status: 'incompatible', score: percentage, reasons: ['Low compatibility score'] }
				}
			} catch (err) {
				console.error('Compatibility check error:', err)
				return { status: 'unknown', score: 0, reasons: [] }
			}
		},
		rankProject(project, query) {
			let score = 0
			const q = (query || '').toLowerCase().trim()
			const title = (project?.title || '').toLowerCase()
			const slug = (project?.slug || '').toLowerCase()
			const description = (project?.description || '').toLowerCase()
			const author = (project?.author || '').toLowerCase()

			// Exact matches get highest priority (50 points)
			if (slug && q && slug === q) score += 50
			if (title && q && title === q) score += 45

			// Fuzzy matching for title (40 points max)
			if (title && q && q.length > 0 && title !== q) {
				if (title.startsWith(q)) {
					const coverageRatio = q.length / title.length
					score += 35 + (5 * coverageRatio) // 35-40 points based on how much is covered
				} else if (title.includes(q)) {
					const coverageRatio = q.length / title.length
					score += 25 + (10 * coverageRatio) // 25-35 points
				} else {
					// Fuzzy matching
					const distance = this.levenshteinDistance(title, q)
					const maxLen = Math.max(title.length, q.length)
					const similarity = 1 - distance / maxLen
					if (similarity > 0.6) {
						score += 30 * similarity
					}
				}
			}

			// Slug fuzzy matching (20 points max)
			if (slug && q && slug !== q && slug.includes(q)) {
				const coverageRatio = q.length / slug.length
				score += 15 + (5 * coverageRatio)
			}

			// Description matching (8 points)
			if (description && q && description.includes(q)) {
				score += 8
			}

			// Author matching (5 points)
			if (author && q && author.includes(q)) {
				score += 5
			}

			// Downloads boost (log scale, 12 points max)
			const dl = Number(project?.downloads || 0)
			score += Math.min(12, Math.log10(dl + 1) * 1.2)

			// Follows boost (6 points max)
			const follows = Number(project?.follows || 0)
			score += Math.min(6, Math.log10(follows + 1) * 0.6)

			// Recency boost (6 points)
			const mod = project?.date_modified ? new Date(project.date_modified) : null
			if (mod && !isNaN(mod)) {
				const days = (Date.now() - mod.getTime()) / (1000 * 60 * 60 * 24)
				if (days <= 30) score += 6
				else if (days <= 90) score += 4
				else if (days <= 180) score += 2
			}

			// Compatibility weighting (18 points max)
			const compat = this.computeCompatibility(project)
			switch (compat.status) {
				case 'compatible':
					score += 18 * (compat.score / 100)
					break
				case 'incompatible':
					score -= 12
					break
				case 'unknown':
					score += 8 * (compat.score / 100)
					break
			}

			return score
		},
		rerankHits(hits, query) {
			return [...hits]
				.map((h) => {
					const score = this.rankProject(h, query)
					return { ...h, __score: score, __matchScore: score }
				})
				.sort((a, b) => {
					if (b.__score !== a.__score) return b.__score - a.__score
					return (b.downloads || 0) - (a.downloads || 0)
				})
		},
		applyFilters(hits) {
			let filtered = [...hits]
			if (this.dependencyFilterCompatibility !== 'all') {
				filtered = filtered.filter((h) => {
					const compat = this.computeCompatibility(h)
					return compat.status === this.dependencyFilterCompatibility
				})
			}

			return filtered
		},
		onFilterChange() {
			if (this.lastSearchQuery) {
				this.searchProjects(this.lastSearchQuery)
			}
		},
		async searchProjects(query) {
			this.lastSearchQuery = query

			if (!query || query.length < 2) {
				this.projectSearchResults = []
				return
			}

			if (this.searchDebounce) {
				clearTimeout(this.searchDebounce)
			}

			this.searchDebounce = setTimeout(async () => {
				this.isSearchingProjects = true
				try {
					let allowedTypes = ['mod', 'plugin', 'resourcepack', 'datapack', 'shader']
					if (this.dependencyFilterType !== 'all') {
						allowedTypes = [this.dependencyFilterType]
					}

					const facets = JSON.stringify([allowedTypes.map((t) => `project_type:${t}`)])
					const baseParams = `query=${encodeURIComponent(query)}&limit=20&index=relevance`
					let results = await useBaseFetch(
						`search?${baseParams}&facets=${encodeURIComponent(facets)}`,
						{},
						true,
					)

					let hits = results.hits || []

					hits = await Promise.all(
						hits.map(async (h) => {
							try {
								const projRef = h.project_id || h.id || h.slug

								let versions = []
								if (projRef) {
									try {
										versions = await useBaseFetch(
											`project/${projRef}/version?limit=50`,
											{},
											true,
										)
									} catch {}
								}
								if (versions && versions.length > 0) {
									let best = versions[0]
									let bestScore = -1
									for (const v of versions) {
										const gvOverlap = (v.game_versions || []).filter((gv) => (this.version?.game_versions || []).includes(gv)).length
										const ldOverlap = (v.loaders || []).filter((ld) => (this.version?.loaders || []).includes(ld)).length
										const s = gvOverlap * 2 + ldOverlap * 3
										if (s > bestScore) {
											best = v
											bestScore = s
										}
									}

									// Expose fields
									h.loaders = best.loaders || h.loaders
									h.game_versions = best.game_versions || h.game_versions

									// Dependency info for badge
									h.hasDependencies = Array.isArray(best.dependencies) && best.dependencies.length > 0
									h.dependencyCount = Array.isArray(best.dependencies) ? best.dependencies.length : 0


								} else {
									try {
										const proj = await useBaseFetch(`project/${projRef}`, {}, true)
										h.loaders = proj?.loaders || h.loaders
										h.game_versions = proj?.game_versions || h.game_versions
									} catch {}
								}
							} catch {
							}
							return h
						}),
					)

					// Rerank and apply filters
					const reranked = this.rerankHits(hits, query)
					this.projectSearchResults = this.applyFilters(reranked)

					// retry without facets
					if (!this.projectSearchResults.length) {
						results = await useBaseFetch(`search?${baseParams}`, {}, true)
						hits = results.hits || []
						// Enrich again on the fallback path
						hits = await Promise.all(
							hits.map(async (h) => {
								try {
									const projRef = h.project_id || h.id || h.slug
									let versions = []
									if (projRef) {
										try {
											versions = await useBaseFetch(
												`project/${projRef}/version?limit=50`,
												{},
												true,
											)
										} catch {}
									}
									if (versions && versions.length > 0) {
										let best = versions[0]
										let bestScore = -1
										for (const v of versions) {
											const gvOverlap = (v.game_versions || []).filter((gv) => (this.version?.game_versions || []).includes(gv)).length
											const ldOverlap = (v.loaders || []).filter((ld) => (this.version?.loaders || []).includes(ld)).length
											const s = gvOverlap * 2 + ldOverlap * 3
											if (s > bestScore) {
												best = v
												bestScore = s
											}
										}
										h.loaders = best.loaders || h.loaders
										h.game_versions = best.game_versions || h.game_versions
										h.hasDependencies = Array.isArray(best.dependencies) && best.dependencies.length > 0
										h.dependencyCount = Array.isArray(best.dependencies) ? best.dependencies.length : 0
									} else {
										try {
											const proj = await useBaseFetch(`project/${projRef}`, {}, true)
											h.loaders = proj?.loaders || h.loaders
											h.game_versions = proj?.game_versions || h.game_versions
										} catch {}
									}
								} catch {}
								return h
							}),
						)
						const reranked = this.rerankHits(hits, query)
						this.projectSearchResults = this.applyFilters(reranked)
					}
				} catch (err) {
					console.error('Error searching projects:', err)
					this.projectSearchResults = []
				} finally {
					this.isSearchingProjects = false
				}
			}, 300)
		},
		onProjectSelect(project) {
			if (project) {
				this.newDependencyId = project.slug || project.project_id
			}
		},
		async fetchTransitiveDependencies(projectId) {
			this.transitiveDependencies = []
			try {
				const versions = await useBaseFetch(`project/${projectId}/version`, {}, true)
				if (!versions || versions.length === 0) return

				let targetVersion = versions.find(
					(v) =>
						v.game_versions.some((gv) => this.version.game_versions.includes(gv)) &&
						v.loaders.some((l) => this.version.loaders.includes(l)),
				)
				if (!targetVersion) targetVersion = versions[0]

				if (!targetVersion.dependencies || targetVersion.dependencies.length === 0) return

				const depProjects = await Promise.all(
					targetVersion.dependencies
						.filter((d) => d.dependency_type === 'required' || d.dependency_type === 'optional')
						.filter((d) => !this.version.dependencies.some((vd) => vd.project_id === d.project_id))
						.map(async (dep) => {
							try {
								const proj = await useBaseFetch(`project/${dep.project_id}`, {}, true)
								return {
									...proj,
									dependency_type: dep.dependency_type,
								}
							} catch {
								return null
							}
						}),
				)

				this.transitiveDependencies = depProjects.filter((p) => p !== null)
			} catch (err) {
				console.error('Error fetching transitive dependencies:', err)
			}
		},
		async addDependencyWithTransitive(projectId, dependencyType) {
			try {
				await this.addDependency('project', projectId, dependencyType)

				for (const dep of this.transitiveDependencies) {
					await this.addDependency('project', dep.slug || dep.project_id, dep.dependency_type, true)
				}

				this.addNotification({
					title: 'Dependencies added',
					text: `Added ${1 + this.transitiveDependencies.length} dependencies successfully.`,
					type: 'success',
				})

				this.selectedProject = null
				this.projectSearchResults = []
				this.transitiveDependencies = []
				this.newDependencyId = ''
			} catch (err) {
				this.addNotification({
					title: 'Error adding dependencies',
					text: err.message || 'Failed to add some dependencies',
					type: 'error',
				})
			}
		},
		async onImageUpload(file) {
			const response = await useImageUpload(file, { context: 'version' })

			this.uploadedImageIds.push(response.id)
			this.uploadedImageIds = this.uploadedImageIds.slice(-10)

			return response.url
		},
		getPreviousLink() {
			if (this.$router.options.history.state.back) {
				if (this.$router.options.history.state.back.includes('/versions')) {
					return this.$router.options.history.state.back
				}
			}
			return `/${this.project.project_type}/${
				this.project.slug ? this.project.slug : this.project.id
			}/versions`
		},
		getPreviousLabel() {
			return this.$router.options.history.state.back &&
				this.$router.options.history.state.back.endsWith('/versions')
				? 'Back to versions'
				: 'All versions'
		},
		acceptFileFromProjectType,
		renderHighlightedString,
		async addDependency(dependencyAddMode, newDependencyId, newDependencyType, hideErrors) {
			try {
				if (dependencyAddMode === 'project') {
					const project = await useBaseFetch(`project/${newDependencyId}`)

					if (this.version.dependencies.some((dep) => project.id === dep.project_id)) {
						this.addNotification({
							title: 'Dependency already added',
							text: 'You cannot add the same dependency twice.',
							type: 'error',
						})
					} else {
						this.version.dependencies.push({
							project,
							project_id: project.id,
							dependency_type: newDependencyType,
							link: `/${project.project_type}/${project.slug ?? project.id}`,
						})

						this.$emit('update:dependencies', {
							projects: this.dependencies.projects.concat([project]),
							versions: this.dependencies.versions,
						})
					}
				} else if (dependencyAddMode === 'version') {
					const version = await useBaseFetch(`version/${this.newDependencyId}`)

					const project = await useBaseFetch(`project/${version.project_id}`)

					if (this.version.dependencies.some((dep) => version.id === dep.version_id)) {
						this.addNotification({
							title: 'Dependency already added',
							text: 'You cannot add the same dependency twice.',
							type: 'error',
						})
					} else {
						this.version.dependencies.push({
							version,
							project,
							version_id: version.id,
							project_id: project.id,
							dependency_type: this.newDependencyType,
							link: `/${project.project_type}/${project.slug ?? project.id}/version/${encodeURI(
								version.version_number,
							)}`,
						})

						this.$emit('update:dependencies', {
							projects: this.dependencies.projects.concat([project]),
							versions: this.dependencies.versions.concat([version]),
						})
					}
				}

				this.newDependencyId = ''
			} catch {
				if (!hideErrors) {
					this.addNotification({
						title: 'Invalid Dependency',
						text: 'The specified dependency could not be found',
						type: 'error',
					})
				}
			}
		},
		async saveEditedVersion() {
			startLoading()

			if (this.fieldErrors) {
				this.showKnownErrors = true

				stopLoading()
				return
			}

			try {
				if (this.newFiles.length > 0) {
					const formData = new FormData()
					const fileParts = this.newFiles.map((f, idx) => `${f.name}-${idx}`)

					formData.append(
						'data',
						JSON.stringify({
							file_types: this.newFileTypes.reduce(
								(acc, x, i) => ({
									...acc,
									[fileParts[i]]: x ? x.value : null,
								}),
								{},
							),
						}),
					)

					for (let i = 0; i < this.newFiles.length; i++) {
						formData.append(fileParts[i], new Blob([this.newFiles[i]]), this.newFiles[i].name)
					}

					await useBaseFetch(`version/${this.version.id}/file`, {
						method: 'POST',
						body: formData,
						headers: {
							'Content-Disposition': formData,
						},
					})
				}

				const body = {
					name: this.version.name || this.version.version_number,
					version_number: this.version.version_number,
					changelog: this.version.changelog,
					version_type: this.version.version_type,
					dependencies: this.version.dependencies,
					game_versions: this.version.game_versions,
					loaders: this.version.loaders,
					primary_file: ['sha1', this.primaryFile.hashes.sha1],
					featured: this.version.featured,
					file_types: this.oldFileTypes.map((x, i) => {
						return {
							algorithm: 'sha1',
							hash: this.version.files[i].hashes.sha1,
							file_type: x ? x.value : null,
						}
					}),
				}

				if (this.project.project_type === 'modpack') {
					delete body.dependencies
				}

				await useBaseFetch(`version/${this.version.id}`, {
					method: 'PATCH',
					body,
				})

				for (const hash of this.deleteFiles) {
					await useBaseFetch(`version_file/${hash}?version_id=${this.version.id}`, {
						method: 'DELETE',
					})
				}

				await this.resetProjectVersions()

				await this.$router.replace(
					`/${this.project.project_type}/${
						this.project.slug ? this.project.slug : this.project.id
					}/version/${encodeURI(
						this.versions.find((x) => x.id === this.version.id).displayUrlEnding,
					)}`,
				)
			} catch (err) {
				this.addNotification({
					title: 'An error occurred',
					text: err.data ? err.data.description : err,
					type: 'error',
				})
				window.scrollTo({ top: 0, behavior: 'smooth' })
			}
			stopLoading()
		},
		reportVersion,
		async createVersion() {
			this.shouldPreventActions = true
			startLoading()
			if (this.fieldErrors) {
				this.showKnownErrors = true
				this.shouldPreventActions = false

				stopLoading()
				return
			}

			try {
				await this.createVersionRaw(this.version)
			} catch (err) {
				this.addNotification({
					title: 'An error occurred',
					text: err.data ? err.data.description : err,
					type: 'error',
				})
				window.scrollTo({ top: 0, behavior: 'smooth' })
			}

			stopLoading()
			this.shouldPreventActions = false
		},
		async createVersionRaw(version) {
			const formData = new FormData()

			const fileParts = this.newFiles.map((f, idx) => `${f.name}-${idx}`)
			if (this.replaceFile) {
				fileParts.unshift(this.replaceFile.name.concat('-primary'))
			}

			if (this.project.project_type === 'resourcepack') {
				version.loaders = ['minecraft']
			}

			const newVersion = {
				project_id: version.project_id,
				file_parts: fileParts,
				version_number: version.version_number,
				version_title: version.name || version.version_number,
				version_body: version.changelog,
				dependencies: version.dependencies,
				game_versions: version.game_versions,
				loaders: version.loaders,
				release_channel: version.version_type,
				featured: version.featured,
				file_types: this.newFileTypes.reduce(
					(acc, x, i) => ({
						...acc,
						[fileParts[this.replaceFile ? i + 1 : i]]: x ? x.value : null,
					}),
					{},
				),
			}

			formData.append('data', JSON.stringify(newVersion))

			if (this.replaceFile) {
				formData.append(
					this.replaceFile.name.concat('-primary'),
					new Blob([this.replaceFile]),
					this.replaceFile.name,
				)
			}

			for (let i = 0; i < this.newFiles.length; i++) {
				formData.append(
					fileParts[this.replaceFile ? i + 1 : i],
					new Blob([this.newFiles[i]]),
					this.newFiles[i].name,
				)
			}

			const data = await useBaseFetch('version', {
				method: 'POST',
				body: formData,
				headers: {
					'Content-Disposition': formData,
				},
			})

			await this.resetProjectVersions()

			await this.$router.push(
				`/${this.project.project_type}/${
					this.project.slug ? this.project.slug : this.project.project_id
				}/version/${data.id}`,
			)
		},
		async deleteVersion() {
			startLoading()

			await useBaseFetch(`version/${this.version.id}`, {
				method: 'DELETE',
			})

			await this.resetProjectVersions()
			await this.$router.replace(`/${this.project.project_type}/${this.project.id}/versions`)
			stopLoading()
		},
		async createDataPackVersion() {
			this.shouldPreventActions = true
			startLoading()
			try {
				const blob = await createDataPackVersion(
					this.project,
					this.version,
					this.primaryFile,
					this.members,
					this.tags.gameVersions,
					this.packageLoaders,
				)

				this.newFiles = []
				this.newFileTypes = []
				this.replaceFile = new File(
					[blob],
					`${this.project.slug}-${this.version.version_number}.jar`,
				)

				await this.createVersionRaw({
					project_id: this.project.id,
					author_id: this.currentMember.user.id,
					name: this.version.name,
					version_number: `${this.version.version_number}+mod`,
					changelog: this.version.changelog,
					version_type: this.version.version_type,
					dependencies: this.version.dependencies,
					game_versions: this.version.game_versions,
					loaders: this.packageLoaders,
					featured: this.version.featured,
				})

				this.$refs.modal_package_mod.hide()

				this.addNotification({
					title: 'Packaging Success',
					text: 'Your data pack was successfully packaged as a mod! Make sure to playtest to check for errors.',
					type: 'success',
				})
			} catch (err) {
				this.addNotification({
					title: 'An error occurred',
					text: err.data ? err.data.description : err,
					type: 'error',
				})
			}
			stopLoading()
			this.shouldPreventActions = false
		},
		async resetProjectVersions() {
			const [versions, dependencies] = await Promise.all([
				useBaseFetch(`project/${this.version.project_id}/version`),
				useBaseFetch(`project/${this.version.project_id}/dependencies`),
				this.resetProject(),
			])

			const newCreatedVersions = this.$computeVersions(versions, this.members)
			this.$emit('update:versions', newCreatedVersions)
			this.$emit('update:dependencies', dependencies)

			return newCreatedVersions
		},
	},
})
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

.dependency-search-select {
	:deep(.multiselect__option) {
		padding: 0;
		min-height: auto;
		overflow: hidden;
	}

	:deep(.multiselect__single) {
		padding: 0;
		margin: 0;
	}

	:deep(.multiselect__content-wrapper) {
		overflow-x: hidden;
	}
}

.dependency-search-option {
	display: flex;
	align-items: center;
	gap: 0.75rem;
	padding: 0.5rem 0.75rem;
	width: 100%;
	min-width: 0;
	max-width: 100%;
}

.dependency-search-info {
	display: flex;
	flex-direction: column;
	gap: 0.25rem;
	flex: 1;
	min-width: 0;
}

.dependency-search-meta {
	font-size: 0.875rem;
	color: var(--color-secondary);
	overflow: hidden;
	text-overflow: ellipsis;
	display: -webkit-box;
	-webkit-line-clamp: 2;
	-webkit-box-orient: vertical;
}


.dependency-search-title-row {
	display: flex;
	justify-content: space-between;
	align-items: flex-start;
	gap: 0.5rem;
	min-width: 0;
}

.dependency-search-title {
	flex: 1;
	min-width: 0;
	overflow: hidden;
	text-overflow: ellipsis;
	font-weight: 600;
	color: var(--color-contrast);
	display: -webkit-box;
	-webkit-line-clamp: 2;
	-webkit-box-orient: vertical;
	/* Add this to ensure it doesn't push the score off */
	max-width: calc(100% - 3.5rem); /* Reserve space for score badge */
}

.dependency-search-score {
	font-size: 0.75rem;
	font-weight: 600;
	color: var(--color-brand);
	background: var(--color-brand-highlight);
	padding: 0.125rem 0.375rem;
	border-radius: 0.25rem;
	white-space: nowrap;
	flex-shrink: 0;
	align-self: flex-start;
	min-width: fit-content;
}

.dependency-search-badges {
	display: flex;
	gap: 0.375rem;
	flex-wrap: wrap;
	margin-top: 0.125rem;
}

.dependency-filters {
	display: flex;
	gap: 0.5rem;
	margin-bottom: 0.75rem;
	flex-wrap: wrap;

	.filter-select {
		flex: 1;
		min-width: 150px;
	}
}

.transitive-dependencies {
	margin-top: 1rem;
	padding: 1rem;
	background: var(--color-bg);
	border-radius: var(--size-rounded-sm);
	border: 1px solid var(--color-divider);

	h5 {
		margin: 0 0 0.75rem 0;
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--color-contrast);
	}

	.transitive-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.transitive-item {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem;
		background: var(--color-raised-bg);
		border-radius: var(--size-rounded-sm);

		:deep(.transitive-badge) {
			font-size: 0.75rem;
		}

		.transitive-title {
			flex: 1;
			font-size: 0.875rem;
			color: var(--color-contrast);
		}
	}
}
</style>
