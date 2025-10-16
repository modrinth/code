import type { Project } from '@modrinth/utils'
import { defineMessage, useVIntl } from '@vintl/vintl'

import type { Nag, NagContext } from '../../types/nags'

const allResolutionTags = ['8x-', '16x', '32x', '48x', '64x', '128x', '256x', '512x+']

const MAX_TAG_COUNT = 8

function getCategories(
	project: Project & { actualProjectType: string },
	tags: {
		categories?: {
			project_type: string
		}[]
	},
) {
	return (
		tags.categories?.filter(
			(category: { project_type: string }) => category.project_type === project.actualProjectType,
		) ?? []
	)
}

export const tagsNags: Nag[] = [
	{
		id: 'too-many-tags',
		title: defineMessage({
			id: 'nags.too-many-tags.title',
			defaultMessage: 'Select accurate tags',
		}),
		description: (context: NagContext) => {
			const { formatMessage } = useVIntl()
			const tagCount =
				context.project.categories.length + (context.project.additional_categories?.length || 0)
			const maxTagCount = MAX_TAG_COUNT

			return formatMessage(
				defineMessage({
					id: 'nags.too-many-tags.description',
					defaultMessage:
						"You've selected {tagCount} tags. Consider reducing to {maxTagCount} or fewer to make sure your project appears in relevant search results.",
				}),
				{
					tagCount,
					maxTagCount,
				},
			)
		},
		status: 'warning',
		shouldShow: (context: NagContext) => {
			const tagCount =
				context.project.categories.length + (context.project.additional_categories?.length || 0)
			return tagCount > MAX_TAG_COUNT
		},
		link: {
			path: 'settings/tags',
			title: defineMessage({
				id: 'nags.edit-tags.title',
				defaultMessage: 'Edit tags',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-tags',
		},
	},
	{
		id: 'multiple-resolution-tags',
		title: defineMessage({
			id: 'nags.multiple-resolution-tags.title',
			defaultMessage: 'Select correct resolution',
		}),
		description: (context: NagContext) => {
			const { formatMessage } = useVIntl()
			const resolutionTags = context.project.categories
				.concat(context.project.additional_categories)
				.filter((tag: string) => allResolutionTags.includes(tag))

			const sortedTags = resolutionTags.toSorted((a, b) => {
				return allResolutionTags.indexOf(a) - allResolutionTags.indexOf(b)
			})

			return formatMessage(
				defineMessage({
					id: 'nags.multiple-resolution-tags.description',
					defaultMessage:
						"You've selected {count} resolution tags ({tags}). Resource packs should typically only have one resolution tag that matches their primary resolution.",
				}),
				{
					count: resolutionTags.length,
					tags: sortedTags
						.join(', ')
						.replace('8x-', '8x or lower')
						.replace('512x+', '512x or higher'),
				},
			)
		},
		status: 'warning',
		shouldShow: (context: NagContext) => {
			if (context.project.project_type !== 'resourcepack') return false

			const resolutionTags = context.project.categories
				.concat(context.project.additional_categories)
				.filter((tag: string) => allResolutionTags.includes(tag))
			return resolutionTags.length > 1
		},
		link: {
			path: 'settings/tags',
			title: defineMessage({
				id: 'nags.edit-tags.title',
				defaultMessage: 'Edit tags',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-tags',
		},
	},
	{
		id: 'all-tags-selected',
		title: defineMessage({
			id: 'nags.all-tags-selected.title',
			defaultMessage: 'Select accurate tags',
		}),
		description: (context: NagContext) => {
			const { formatMessage } = useVIntl()
			const categoriesForProjectType = getCategories(
				context.project as Project & { actualProjectType: string },
				context.tags,
			)
			const totalAvailableTags = categoriesForProjectType.length

			return formatMessage(
				defineMessage({
					id: 'nags.all-tags-selected.description',
					defaultMessage:
						"You've selected all {totalAvailableTags} available tags. This defeats the purpose of tags, which are meant to help users find relevant projects. Please select only the tags that are relevant to your project.",
				}),
				{
					totalAvailableTags,
				},
			)
		},
		status: 'required',
		shouldShow: (context: NagContext) => {
			const categoriesForProjectType = getCategories(
				context.project as Project & { actualProjectType: string },
				context.tags,
			)
			const totalSelectedTags =
				context.project.categories.length + (context.project.additional_categories?.length || 0)
			return (
				totalSelectedTags === categoriesForProjectType.length &&
				context.project.project_type !== 'project'
			)
		},
		link: {
			path: 'settings/tags',
			title: defineMessage({
				id: 'nags.edit-tags.title',
				defaultMessage: 'Edit tags',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-tags',
		},
	},
]
