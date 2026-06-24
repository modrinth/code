import { defineMessage, formatProjectTypeSentence, useVIntl } from '@modrinth/ui'

import type { Nag, NagContext } from '../../types/nags'

export const coreNags: Nag[] = [
	{
		id: 'moderator-feedback',
		title: defineMessage({
			id: 'nags.moderator-feedback.title',
			defaultMessage: 'Review feedback',
		}),
		description: defineMessage({
			id: 'nags.moderator-feedback.description',
			defaultMessage:
				'Review and address all concerns from the moderation team before resubmitting.',
		}),
		status: 'warning',
		shouldShow: (context: NagContext) =>
			context.tags.rejectedStatuses.includes(context.project.status),
		link: {
			path: 'moderation',
			title: defineMessage({
				id: 'nags.moderation.title',
				defaultMessage: 'Visit moderation thread',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-moderation',
		},
	},
	{
		id: 'upload-version',
		title: defineMessage({
			id: 'nags.upload-version.title',
			defaultMessage: 'Upload a version',
		}),
		description: defineMessage({
			id: 'nags.upload-version.description',
			defaultMessage: 'At least one version is required for a project to be submitted for review.',
		}),
		status: 'required',
		shouldShow: (context: NagContext) =>
			context.projectV3?.versions?.length < 1 && !context.projectV3?.minecraft_server,
		link: {
			path: 'settings/versions',
			title: defineMessage({
				id: 'nags.versions.title',
				defaultMessage: 'Visit versions page',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-versions',
		},
	},
	{
		id: 'add-description',
		title: defineMessage({
			id: 'nags.add-description.title',
			defaultMessage: 'Add a description',
		}),
		description: defineMessage({
			id: 'nags.add-description.description',
			defaultMessage:
				"A description that clearly describes the project's purpose and function is required.",
		}),
		status: 'required',
		shouldShow: (context: NagContext) => context.project.body === '',
		link: {
			path: 'settings/description',
			title: defineMessage({
				id: 'nags.settings.description.title',
				defaultMessage: 'Visit description settings',
			}),
			shouldShow: (context: NagContext) =>
				context.currentRoute !== 'type-project-settings-description',
		},
	},
	{
		id: 'add-icon',
		title: defineMessage({
			id: 'nags.add-icon.title',
			defaultMessage: 'Add an icon',
		}),
		description: defineMessage({
			id: 'nags.add-icon.description',
			defaultMessage:
				'Adding a unique, relevant, and engaging icon makes your project identifiable and helps it stand out.',
		}),
		status: 'suggestion',
		shouldShow: (context: NagContext) => !context.project.icon_url,
		link: {
			path: 'settings',
			title: defineMessage({
				id: 'nags.settings.title',
				defaultMessage: 'Visit general settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings',
		},
	},
	{
		id: 'upload-gallery-image',
		title: defineMessage({
			id: 'nags.upload-gallery-image.title',
			defaultMessage: 'Upload a gallery image',
		}),
		description: (context: NagContext) => {
			const { formatMessage } = useVIntl()

			return formatMessage(
				defineMessage({
					id: 'nags.upload-gallery-image.description',
					defaultMessage:
						'At least one gallery image is required to showcase the content of your {type}.',
				}),
				{
					type:
						context.project.project_type === 'resourcepack'
							? formatMessage(
									defineMessage({
										id: 'nags.upload-gallery-image.resourcepack-type',
										defaultMessage:
											'resource pack, except for audio or localization packs. If this describes your pack, please select the appropriate tag',
									}),
								)
							: formatProjectTypeSentence(formatMessage, context.project.project_type),
				},
			)
		},
		status: 'required',
		shouldShow: (context: NagContext) => {
			return (
				(context.project.project_type === 'resourcepack' ||
					context.project.project_type === 'shader') &&
				(!context.project.gallery || context.project.gallery?.length === 0) &&
				!(
					context.project.categories.includes('audio') ||
					context.project.additional_categories.includes('audio') ||
					context.project.categories.includes('locale') ||
					context.project.additional_categories.includes('locale')
				)
			)
		},
		link: {
			path: 'settings/gallery',
			title: defineMessage({
				id: 'nags.gallery.title',
				defaultMessage: 'Visit gallery page',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-gallery',
		},
	},
	{
		id: 'feature-gallery-image',
		title: defineMessage({
			id: 'nags.feature-gallery-image.title',
			defaultMessage: 'Feature a gallery image',
		}),
		description: defineMessage({
			id: 'nags.feature-gallery-image.description',
			defaultMessage:
				'The featured gallery image is often how your project makes its first impression.',
		}),
		status: 'suggestion',
		shouldShow: (context: NagContext) => {
			if (context.projectV3?.minecraft_server) return false
			const featuredGalleryImage = context.project.gallery?.find((img) => img.featured)
			return context.project?.gallery?.length === 0 || !featuredGalleryImage
		},
		link: {
			path: 'settings/gallery',
			title: defineMessage({
				id: 'nags.gallery.title',
				defaultMessage: 'Visit gallery page',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-gallery',
		},
	},
	{
		id: 'select-license',
		title: defineMessage({
			id: 'nags.select-license.title',
			defaultMessage: 'Select a license',
		}),
		description: (context: NagContext) => {
			const { formatMessage } = useVIntl()

			return formatMessage(
				defineMessage({
					id: 'nags.select-license.description',
					defaultMessage: 'Select the license your {type} is distributed under.',
				}),
				{
					type: formatProjectTypeSentence(formatMessage, context.project.project_type),
				},
			)
		},
		status: 'required',
		shouldShow: (context: NagContext) =>
			context.project.license.id === 'LicenseRef-Unknown' && !context.projectV3?.minecraft_server,
		link: {
			path: 'settings/license',
			title: defineMessage({
				id: 'nags.settings.license.title',
				defaultMessage: 'Visit license settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings-license',
		},
	},
]
