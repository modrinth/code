import type { Labrinth } from '@modrinth/api-client'
import { defineMessages } from '@modrinth/ui/i18n'
import { formatCategory } from '@modrinth/ui/src/utils/tag-messages.ts'

import type { Nag, ProjectValidationContext } from '../../types/nags.ts'
import { evaluateRules } from '../evaluate-rules.ts'
import { toNags } from '../to-nags.ts'
import type { ValidationRuleSet } from '../types.ts'

const messages = defineMessages({
	selectTags: {
		id: 'nags.select-tags.title',
		defaultMessage: 'Select tags',
	},
	selectTagsDescription: {
		id: 'nags.select-tags.description',
		defaultMessage:
			'Select the tags that correctly apply to your project to help the right users find it.',
	},
	selectAccurateTags: {
		id: 'nags.too-many-tags.title',
		defaultMessage: 'Select accurate tags',
	},
	selectAccurateServerTags: {
		id: 'nags.too-many-tags-server.title',
		defaultMessage: 'Select accurate tags',
	},
	selectAllTags: {
		id: 'nags.all-tags-selected.title',
		defaultMessage: 'Select accurate tags',
	},
	tooManyTags: {
		id: 'nags.too-many-tags.description',
		defaultMessage:
			"You've selected {tagCount, plural, one {# tag} other {# tags}}. Consider reducing to {maxTagCount} or fewer to make sure your project appears in relevant search results.",
	},
	tooManyServerTags: {
		id: 'nags.too-many-tags-server.description',
		defaultMessage:
			"You've selected {tagCount, plural, one {# tag} other {# tags}}. Please reduce to {maxTagCount} or fewer to make sure your server appears in relevant search results.",
	},
	selectResolution: {
		id: 'nags.multiple-resolution-tags.title',
		defaultMessage: 'Select correct resolution',
	},
	multipleResolutionTags: {
		id: 'nags.multiple-resolution-tags.description',
		defaultMessage:
			"You've selected {count, plural, one {# resolution tag} other {# resolution tags}} ({tags}). Resource packs should typically only have one resolution tag that matches their primary resolution.",
	},
	allTagsSelected: {
		id: 'nags.all-tags-selected.description',
		defaultMessage:
			"You've selected all {totalAvailableTags, plural, one {# available tag} other {# available tags}}. This defeats the purpose of tags, which are meant to help users find relevant projects. Please select only the tags that are relevant to your project.",
	},
	editTags: {
		id: 'nags.edit-tags.title',
		defaultMessage: 'Edit tags',
	},
})

export const allResolutionTags = ['8x-', '16x', '32x', '48x', '64x', '128x', '256x', '512x+']
export const MAX_TAG_COUNT = 8
export const MAX_TAG_COUNT_SERVER = 18

function getCategories(
	project: Labrinth.Projects.v2.Project & { actualProjectType: string },
	tags: ProjectValidationContext['tags'],
) {
	return (
		tags.categories?.filter((category) => category.project_type === project.actualProjectType) ?? []
	)
}

function getSelectedTagCount(context: ProjectValidationContext) {
	return context.project.categories.length + (context.project.additional_categories?.length ?? 0)
}

function getResolutionTags(context: ProjectValidationContext) {
	return context.project.categories
		.concat(context.project.additional_categories ?? [])
		.filter((tag) => allResolutionTags.includes(tag))
		.toSorted((a, b) => allResolutionTags.indexOf(a) - allResolutionTags.indexOf(b))
}

export const projectTagsValidationRules = {
	'select-tags': {
		severity: 'suggestion',
		evaluate: (context) => ({
			valid: context.project.versions.length === 0 || context.project.categories.length > 0,
		}),
		presentation: {
			message: messages.selectTagsDescription,
			nag: { title: messages.selectTags, destination: 'tags' },
		},
	},
	'too-many-tags': {
		severity: 'warning',
		evaluate: (context) => {
			const tagCount = getSelectedTagCount(context)
			const tooMany =
				!context.projectV3.minecraft_java_server &&
				!context.projectV3.minecraft_server &&
				tagCount > MAX_TAG_COUNT
			return tooMany
				? { valid: false, values: { tagCount, maxTagCount: MAX_TAG_COUNT } }
				: { valid: true }
		},
		presentation: {
			message: messages.tooManyTags,
			nag: {
				title: messages.selectAccurateTags,
				destination: 'tags',
				linkTitle: messages.editTags,
			},
		},
	},
	'too-many-tags-server': {
		severity: 'error',
		evaluate: (context) => {
			const tagCount = getSelectedTagCount(context)
			return context.projectV3.minecraft_server && tagCount > MAX_TAG_COUNT_SERVER
				? { valid: false, values: { tagCount, maxTagCount: MAX_TAG_COUNT_SERVER } }
				: { valid: true }
		},
		presentation: {
			message: messages.tooManyServerTags,
			nag: {
				title: messages.selectAccurateServerTags,
				destination: 'tags',
				linkTitle: messages.editTags,
			},
		},
	},
	'multiple-resolution-tags': {
		severity: 'warning',
		evaluate: (context) => {
			const resolutionTags = getResolutionTags(context)
			return context.project.project_type === 'resourcepack' && resolutionTags.length > 1
				? {
						valid: false,
						values: { count: resolutionTags.length, tags: resolutionTags.join('|') },
					}
				: { valid: true }
		},
		presentation: {
			message: messages.multipleResolutionTags,
			nag: {
				title: messages.selectResolution,
				destination: 'tags',
				linkTitle: messages.editTags,
				formatValues: (values, formatMessage) => ({
					count: values.count,
					tags: String(values.tags)
						.split('|')
						.map((tag) => formatCategory(formatMessage, tag))
						.join(', '),
				}),
			},
		},
	},
	'all-tags-selected': {
		severity: 'error',
		evaluate: (context) => {
			const categories = getCategories(
				context.project as Labrinth.Projects.v2.Project & { actualProjectType: string },
				context.tags,
			)
			const totalAvailableTags = categories.length
			const allSelected =
				getSelectedTagCount(context) === totalAvailableTags &&
				context.project.project_type !== 'project'
			return allSelected ? { valid: false, values: { totalAvailableTags } } : { valid: true }
		},
		presentation: {
			message: messages.allTagsSelected,
			nag: {
				title: messages.selectAllTags,
				destination: 'tags',
				linkTitle: messages.editTags,
			},
		},
	},
} satisfies ValidationRuleSet<ProjectValidationContext>

export function getTagsNags(context: ProjectValidationContext): Nag[] {
	return toNags(evaluateRules(context, projectTagsValidationRules))
}
