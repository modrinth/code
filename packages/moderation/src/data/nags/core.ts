import { formatProjectType } from '@modrinth/utils'
import { defineMessage, useVIntl } from '@vintl/vintl'

import type { Nag, NagContext } from '../../types/nags'

export const coreNags: Nag[] = [
	{
		id: 'moderator-feedback',
		title: defineMessage({
			id: 'nags.moderator-feedback.title',
			defaultMessage: 'Review moderator feedback',
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
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-moderation',
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
		shouldShow: (context: NagContext) => context.versions.length < 1,
		link: {
			path: 'versions',
			title: defineMessage({
				id: 'nags.versions.title',
				defaultMessage: 'Visit versions page',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-versions',
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
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
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
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
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
			const projectType = formatProjectType(context.project.project_type).toLowerCase()
			let msg = ''
			if (context.project.project_type === 'resourcepack') {
				msg =
					', except for audio or localization packs. If this describes your pack, please select the appropriate tag'
			}
			const resourcepackMessage = msg

			return formatMessage(
				defineMessage({
					id: 'nags.upload-gallery-image.description',
					defaultMessage:
						'At least one gallery image is required to showcase the content of your {type}{resourcepackMessage}.',
				}),
				{
					type: projectType,
					resourcepackMessage: resourcepackMessage,
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
			path: 'gallery',
			title: defineMessage({
				id: 'nags.gallery.title',
				defaultMessage: 'Visit gallery page',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-gallery',
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
			const featuredGalleryImage = context.project.gallery?.find((img) => img.featured)
			return context.project?.gallery?.length === 0 || !featuredGalleryImage
		},
		link: {
			path: 'gallery',
			title: defineMessage({
				id: 'nags.gallery.title',
				defaultMessage: 'Visit gallery page',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-gallery',
		},
	},
	{
		id: 'select-tags',
		title: defineMessage({
			id: 'nags.select-tags.title',
			defaultMessage: 'Select tags',
		}),
		description: defineMessage({
			id: 'nags.select-tags.description',
			defaultMessage:
				'Select the tags that correctly apply to your project to help the right users find it.',
		}),
		status: 'suggestion',
		shouldShow: (context: NagContext) =>
			context.project.versions.length > 0 && context.project.categories.length < 1,
		link: {
			path: 'settings/tags',
			title: defineMessage({
				id: 'nags.settings.tags.title',
				defaultMessage: 'Visit tag settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-tags',
		},
	},
	{
		id: 'add-links',
		title: defineMessage({
			id: 'nags.add-links.title',
			defaultMessage: 'Add external links',
		}),
		description: defineMessage({
			id: 'nags.add-links.description',
			defaultMessage:
				'Add any relevant links targeted outside of Modrinth, such as source code, an issue tracker, or a Discord invite.',
		}),
		status: 'suggestion',
		shouldShow: (context: NagContext) =>
			!(
				context.project.issues_url ||
				context.project.source_url ||
				context.project.wiki_url ||
				context.project.discord_url ||
				context.project.donation_urls.length > 0
			),
		link: {
			path: 'settings/links',
			title: defineMessage({
				id: 'nags.settings.links.title',
				defaultMessage: 'Visit links settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-links',
		},
	},
	{
		id: 'select-environments',
		title: defineMessage({
			id: 'nags.select-environments.title',
			defaultMessage: 'Select environments',
		}),
		description: (context: NagContext) => {
			const { formatMessage } = useVIntl()

			return formatMessage(
				defineMessage({
					id: 'nags.select-environments.description',
					defaultMessage: `Select the environments your {projectType} functions on.`,
				}),
				{
					projectType: formatProjectType(context.project.project_type).toLowerCase(),
				},
			)
		},
		status: 'required',
		shouldShow: (context: NagContext) => {
			const excludedTypes = ['resourcepack', 'plugin', 'shader', 'datapack']
			return (
				context.project.versions.length > 0 &&
				!excludedTypes.includes(context.project.project_type) &&
				(context.project.client_side === 'unknown' ||
					context.project.server_side === 'unknown' ||
					(context.project.client_side === 'unsupported' &&
						context.project.server_side === 'unsupported'))
			)
		},
		link: {
			path: 'settings/environment',
			title: defineMessage({
				id: 'nags.settings.environments.title',
				defaultMessage: 'Visit environment settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-environment',
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
					defaultMessage: 'Select the license your {projectType} is distributed under.',
				}),
				{
					projectType: formatProjectType(context.project.project_type).toLowerCase(),
				},
			)
		},
		status: 'required',
		shouldShow: (context: NagContext) => context.project.license.id === 'LicenseRef-Unknown',
		link: {
			path: 'settings/license',
			title: defineMessage({
				id: 'nags.settings.license.title',
				defaultMessage: 'Visit license settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-license',
		},
	},
]
