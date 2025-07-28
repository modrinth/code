import type { Nag, NagContext } from '../../types/nags'
import { formatProjectType } from '@modrinth/utils'
import { useVIntl, defineMessage } from '@vintl/vintl'

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
        'Review any feedback from moderators regarding your project before resubmitting.',
    }),
    status: 'suggestion',
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
    shouldShow: (context: NagContext) =>
      context.project.body === '' || context.project.body.startsWith('# Placeholder description'),
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
        'Your project should have a nice-looking icon to uniquely identify your project at a glance.',
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
    id: 'feature-gallery-image',
    title: defineMessage({
      id: 'nags.feature-gallery-image.title',
      defaultMessage: 'Feature a gallery image',
    }),
    description: defineMessage({
      id: 'nags.feature-gallery-image.description',
      defaultMessage: 'Featured gallery images may be the first impression of many users.',
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
      defaultMessage: 'Select all tags that apply to your project.',
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
        'Add any relevant links targeted outside of Modrinth, such as sources, issues, or a Discord invite.',
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
      defaultMessage: 'Select supported environments',
    }),
    description: (context: NagContext) => {
      const { formatMessage } = useVIntl()

      return formatMessage(
        defineMessage({
          id: 'nags.select-environments.description',
          defaultMessage: `Select if the {projectType} functions on the client-side and/or server-side.`,
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
      path: 'settings',
      title: defineMessage({
        id: 'nags.settings.environments.title',
        defaultMessage: 'Visit general settings',
      }),
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'select-license',
    title: defineMessage({
      id: 'nags.select-license.title',
      defaultMessage: 'Select license',
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
