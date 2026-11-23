<script setup lang="ts">
import { BoxIcon, PlusIcon } from '@modrinth/assets'
import {
	Button,
	commonProjectSettingsMessages,
	ConfirmLeaveModal,
	defineMessages,
	EmptyState,
	formatCategory,
	formatCategoryHeader,
	formatProjectTypeSentence,
	injectProjectPageContext,
	type MessageDescriptor,
	sortProjectTypes,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { capitalizeString, isAdmin, sortedCategories, TeamMemberPermission } from '@modrinth/utils'
import { computed, useTemplateRef } from 'vue'

import CreateProjectVersionModal from '~/components/ui/create-project-version/CreateProjectVersionModal.vue'
import ProjectIssueCard from '~/components/ui/project-issue-card/index.vue'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { useProjectIssueFieldAction } from '~/composables/project-issue-field-action'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'
import { isPermission } from '~/utils/permissions'
import { projectCopyType } from '~/utils/project-messages'

import SettingsSection from '../settings-section.vue'
import { versionActionMessages } from '../versions/messages'
import FeaturedTags from './featured-tags.vue'
import CategoryPicker from './tag-picker.vue'
import type { Category, CategoryGroup } from './types'

const MAX_FEATURED_TAGS = 3
const MAX_SERVER_TAGS = 18

const SHARED_CATEGORY_PROJECT_TYPES: Record<string, string> = {
	plugin: 'mod',
	datapack: 'mod',
}

const tags = useGeneratedState()
const { formatMessage, locale } = useVIntl()

const messages = defineMessages({
	uploadVersionFirstHeading: {
		id: 'project.settings.tags.upload-version-first.heading',
		defaultMessage: 'Upload a version first',
	},
	uploadVersionFirstDescription: {
		id: 'project.settings.tags.upload-version-first.description',
		defaultMessage: "Come back once you've uploaded your first version",
	},
	description: {
		id: 'project.settings.tags.discovery-help',
		defaultMessage:
			'Help users find your {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}} when filtering in Discovery on Modrinth or the Modrinth App.',
	},
	gameplay: {
		id: 'project.settings.tags.group-title.gameplay',
		defaultMessage: '{showType, select, yes {{types} gameplay} other {Gameplay}}',
	},
	resolutionsDescription: {
		id: 'project.settings.tags.resolutions-description',
		defaultMessage: 'Select the resolution(s) of textures in your {type}.',
	},
	performanceImpactDescription: {
		id: 'project.settings.tags.performance-impact-description',
		defaultMessage:
			'Select the realistic performance impact of your {type}. Select multiple if the {type} is configurable to different levels of performance impact.',
	},
	tooManyTagsServerHardWarning: {
		id: 'project.settings.tags.server-limit-warning',
		defaultMessage:
			"You've selected {count} tags. Please reduce to {limit, number} or fewer to keep your server focused and easier to discover.",
	},
	multipleResolutionTagsWarning: {
		id: 'project.settings.tags.multiple-resolution-tags-warning',
		defaultMessage:
			"You've selected {count} resolution tags ({tags}). Resource packs should typically only have one resolution tag.",
	},
	allTagsSelectedWarning: {
		id: 'project.settings.tags.all-tags-selected-warning',
		defaultMessage:
			"You've selected all {count} available tags. Please select only the tags that truly apply to your project.",
	},
})

const groupTitleMessages: Record<string, MessageDescriptor> = defineMessages({
	categories: {
		id: 'project.settings.tags.group-title.categories',
		defaultMessage: '{showType, select, yes {{types} categories} other {Categories}}',
	},
	features: {
		id: 'project.settings.tags.group-title.features',
		defaultMessage: '{showType, select, yes {{types} features} other {Features}}',
	},
	resolutions: {
		id: 'project.settings.tags.group-title.resolutions',
		defaultMessage: '{showType, select, yes {{types} resolutions} other {Resolutions}}',
	},
	'performance impact': {
		id: 'project.settings.tags.group-title.performance-impact',
		defaultMessage:
			'{showType, select, yes {{types} performance impact} other {Performance impact}}',
	},
})

const groupDescriptionMessages: Record<string, MessageDescriptor> = {
	resolutions: messages.resolutionsDescription,
	'performance impact': messages.performanceImpactDescription,
}

const { projectV2: project, projectV3, patchProject, currentMember } = injectProjectPageContext()

const formatCategoryName = (categoryName: string) => formatCategory(formatMessage, categoryName)

const isServerProject = computed(() => projectV3.value?.minecraft_server != null)

const canSelectTags = computed(() => project.value.versions.length > 0 || isServerProject.value)
const canUploadVersion = computed(() => {
	const member = currentMember.value
	return (
		!!member?.accepted &&
		(isAdmin(member.user) ||
			('staffOnly' in member && member.staffOnly === true) ||
			isPermission(member.permissions, TeamMemberPermission.UPLOAD_VERSION))
	)
})
const createProjectVersionModal = useTemplateRef('create-project-version-modal')

const projectTypes = computed(() => {
	if (isServerProject.value) {
		return ['minecraft_java_server']
	}

	const types = projectV3.value?.project_types?.length
		? projectV3.value.project_types
		: [project.value.actualProjectType]

	return sortProjectTypes(new Set(types))
})

const allCategories = computed(
	() => sortedCategories(tags.value, formatCategoryName, locale.value) as Category[],
)

const projectTypesByCategoryList = computed(() => {
	const lists = new Map<string, string[]>()
	for (const projectType of projectTypes.value) {
		const source = SHARED_CATEGORY_PROJECT_TYPES[projectType] ?? projectType
		lists.set(source, [...(lists.get(source) ?? []), projectType])
	}
	return lists
})

const projectTypeListFormatter = computed(
	() => new Intl.ListFormat(locale.value, { style: 'long', type: 'conjunction' }),
)

function formatProjectTypeName(type: string) {
	return formatProjectTypeSentence(
		formatMessage,
		type === 'minecraft_java_server' ? 'server' : type,
	)
}

function formatGroupTitle(header: string, types: string[]) {
	const message =
		header === 'categories' &&
		types.every((type) => (SHARED_CATEGORY_PROJECT_TYPES[type] ?? type) === 'mod')
			? messages.gameplay
			: groupTitleMessages[header]
	if (!message) {
		return formatCategoryHeader(formatMessage, header)
	}

	const showType = projectTypes.value.length > 1
	return formatMessage(message, {
		showType: showType ? 'yes' : 'other',
		types: showType
			? capitalizeString(projectTypeListFormatter.value.format(types.map(formatProjectTypeName)))
			: '',
	})
}

function formatGroupDescription(header: string, types: string[]) {
	const message = groupDescriptionMessages[header]
	if (!message) {
		return undefined
	}

	// listing out every type gets unwieldy, fallback to the generic term when there's more than one
	return formatMessage(message, {
		type: formatProjectTypeName(types.length > 1 ? 'project' : types[0]),
	})
}

const categorySections = computed(() => {
	const sections: {
		categoryList: string
		header: string
		types: string[]
		categories: Category[]
	}[] = []

	for (const [categoryList, types] of projectTypesByCategoryList.value) {
		const byHeader = new Map<string, Category[]>()
		for (const category of allCategories.value) {
			if (category.project_type !== categoryList) continue
			byHeader.set(category.header, [...(byHeader.get(category.header) ?? []), category])
		}

		for (const [header, categories] of byHeader) {
			sections.push({
				categoryList,
				header,
				types,
				categories:
					header === 'minecraft_server_features' ? withPokemonFirst(categories) : categories,
			})
		}
	}

	return sections
})

const categoryGroups = computed<CategoryGroup[]>(() =>
	categorySections.value.map((section) => ({
		id: `${section.categoryList}-${section.header}`,
		title: formatGroupTitle(section.header, section.types),
		description: formatGroupDescription(section.header, section.types),
		categories: section.categories,
	})),
)

function withPokemonFirst(categories: Category[]) {
	return categories.slice().sort((a, b) => {
		if (a.name === 'pokemon') return -1
		if (b.name === 'pokemon') return 1
		return 0
	})
}

const availableTags = computed(() => [
	...new Set(categorySections.value.flatMap((section) => section.categories.map((x) => x.name))),
])

function hasSameTags(a: string[], b: string[]) {
	return a.length === b.length && a.every((tag) => b.includes(tag))
}

const {
	saved,
	current,
	saving,
	hasChanges,
	reset: resetForm,
	save: saveForm,
} = useSavable(
	() => ({
		selectedTags: availableTags.value.filter(
			(tag) =>
				project.value.categories.includes(tag) || project.value.additional_categories.includes(tag),
		),
		featuredTags: availableTags.value.filter((tag) => project.value.categories.includes(tag)),
	}),
	async () => {
		const featuredTags = current.value.featuredTags
		const additionalCategories = current.value.selectedTags.filter(
			(tag) => !featuredTags.includes(tag),
		)

		const data: Record<string, string[]> = {}

		if (!hasSameTags(featuredTags, project.value.categories)) {
			data.categories = featuredTags
		}

		if (!hasSameTags(additionalCategories, project.value.additional_categories)) {
			data.additional_categories = additionalCategories
		}

		await patchProject(data, false, true)
	},
)

const { confirmLeaveModal } = usePageLeaveSafety(hasChanges)

const saveValidation = useProjectSaveValidation(() => current.value)
const serverTagLimitReached = computed(
	() => isServerProject.value && current.value.selectedTags.length >= MAX_SERVER_TAGS,
)
const serverTagLimitExceeded = computed(
	() => isServerProject.value && current.value.selectedTags.length > MAX_SERVER_TAGS,
)
const serverTagLimitMessage = computed(() =>
	serverTagLimitExceeded.value
		? {
				severity: 'error' as const,
				message: messages.tooManyTagsServerHardWarning,
				values: { count: current.value.selectedTags.length, limit: MAX_SERVER_TAGS },
			}
		: null,
)
const hasPermission = computed(
	() =>
		isAdmin(currentMember.value?.user) ||
		((currentMember.value?.permissions ?? 0) & (1 << 2)) !== 0,
)
const canSave = computed(
	() =>
		hasPermission.value &&
		!serverTagLimitExceeded.value &&
		current.value.featuredTags.length <= MAX_FEATURED_TAGS &&
		!saveValidation.hasErrors.value,
)

const tagsIssueAction = useProjectIssueFieldAction({
	draft: () => ({
		categories: [...current.value.featuredTags],
		additional_categories: current.value.selectedTags.filter(
			(tag) => !current.value.featuredTags.includes(tag),
		),
	}),
	canSave: () =>
		canSave.value &&
		(isAdmin(currentMember.value?.user) ||
			((currentMember.value?.permissions ?? 0) & (1 << 2)) !== 0),
	saving,
	validation: saveValidation,
	save: async () => {
		const categories = [...current.value.featuredTags]
		const additional_categories = current.value.selectedTags.filter(
			(tag) => !categories.includes(tag),
		)
		if (
			!hasSameTags(categories, project.value.categories) ||
			!hasSameTags(additional_categories, project.value.additional_categories)
		) {
			await patchProject({ categories, additional_categories }, true, true)
		}
	},
})

async function save() {
	if (!canSave.value || saving.value) return
	const submittedState = saveValidation.snapshot()
	try {
		await saveForm()
		saveValidation.clear()
	} catch (error) {
		if (!saveValidation.capture(error, submittedState)) throw error
	}
}

function reset() {
	resetForm()
	saveValidation.clear()
}

const savedTagValidation = useProjectNagMessages('tags')
const tagValidation = computed(() =>
	serverTagLimitExceeded.value
		? savedTagValidation.value.filter((message) => message.code !== 'too-many-tags-server')
		: savedTagValidation.value,
)

function toggleTagRaw(selection: string[], tag: string) {
	if (selection.includes(tag)) {
		return selection.filter((x) => x !== tag)
	}
	return availableTags.value.filter((x) => x === tag || selection.includes(x))
}

const toggleTag = (tag: string) => {
	if (
		!hasPermission.value ||
		saving.value ||
		(serverTagLimitReached.value && !current.value.selectedTags.includes(tag))
	)
		return
	current.value.selectedTags = toggleTagRaw(current.value.selectedTags, tag)
	if (!current.value.selectedTags.includes(tag)) {
		current.value.featuredTags = current.value.featuredTags.filter((x) => x !== tag)
	}
}

const toggleFeatured = (tag: string) => {
	if (
		!hasPermission.value ||
		saving.value ||
		(!current.value.featuredTags.includes(tag) &&
			current.value.featuredTags.length >= MAX_FEATURED_TAGS)
	)
		return
	current.value.featuredTags = toggleTagRaw(current.value.featuredTags, tag)
}
</script>
<template>
	<div>
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<CreateProjectVersionModal
			v-if="!canSelectTags && currentMember"
			ref="create-project-version-modal"
			:enable-drop-area="false"
		/>
		<EmptyState
			v-if="!canSelectTags"
			:icon="BoxIcon"
			:heading="formatMessage(messages.uploadVersionFirstHeading)"
			:description="formatMessage(messages.uploadVersionFirstDescription)"
		>
			<template v-if="currentMember" #actions>
				<Button
					type="colored"
					color="brand"
					size="lg"
					:disabled="!canUploadVersion"
					@click="createProjectVersionModal?.openCreateVersionModal()"
				>
					<PlusIcon /> {{ formatMessage(versionActionMessages.create) }}
				</Button>
			</template>
		</EmptyState>
		<SettingsSection
			v-else
			:title="formatMessage(commonProjectSettingsMessages.tags)"
			:subtitle="formatMessage(messages.description, { contentType: projectCopyType(projectV3) })"
		>
			<div class="mb-4 flex flex-col gap-2.5 empty:hidden">
				<ProjectIssueCard target="remove_tags" :field-action="tagsIssueAction" />
				<ValidationMessage :check="serverTagLimitMessage" :debounce="0" />
				<ValidationMessage
					:check="tagValidation"
					:project-field="JSON.stringify(saved)"
					:current-field="JSON.stringify(current)"
				/>
				<ValidationMessage :check="saveValidation.forField('tags')" />
				<ValidationMessage :check="saveValidation.withoutFields(['tags'])" />
			</div>
			<div class="flex flex-col gap-8">
				<CategoryPicker
					:groups="categoryGroups"
					:selected-tags="current.selectedTags"
					:featured-tags="current.featuredTags"
					:disabled="!hasPermission || saving"
					:limit-reached="serverTagLimitReached"
					@toggle="toggleTag"
				/>
				<FeaturedTags
					:project="project"
					:selected-tags="current.selectedTags"
					:featured-tags="current.featuredTags"
					:max-featured-tags="MAX_FEATURED_TAGS"
					:content-type="projectCopyType(projectV3)"
					:disabled="!hasPermission || saving"
					@toggle="toggleFeatured"
				/>
			</div>
		</SettingsSection>
		<UnsavedChangesPopup
			:original="saved"
			:modified="current"
			:saving="saving"
			:can-save="canSave"
			@reset="reset"
			@save="save"
		/>
	</div>
</template>
