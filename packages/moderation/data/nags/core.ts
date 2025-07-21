import type { Nag, NagContext } from '../../types/nags'
import { formatProjectType } from '@modrinth/utils'
import { useVIntl } from '@vintl/vintl'

import messages from './core.i18n'

export const coreNags: Nag[] = [
  {
    id: 'moderator-feedback',
    title: messages.moderatorFeedbackTitle,
    description: messages.moderatorFeedbackDescription,
    status: 'suggestion',
    shouldShow: (context: NagContext) =>
      context.tags.rejectedStatuses.includes(context.project.status),
    link: {
      path: 'moderation',
      title: messages.moderationTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-moderation',
    },
  },
  {
    id: 'upload-version',
    title: messages.uploadVersionTitle,
    description: messages.uploadVersionDescription,
    status: 'required',
    shouldShow: (context: NagContext) => context.versions.length < 1,
    link: {
      path: 'versions',
      title: messages.versionsTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-versions',
    },
  },
  {
    id: 'add-description',
    title: messages.addDescriptionTitle,
    description: messages.addDescriptionDescription,
    status: 'required',
    shouldShow: (context: NagContext) =>
      context.project.body === '' || context.project.body.startsWith('# Placeholder description'),
    link: {
      path: 'settings/description',
      title: messages.settingsDescriptionTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
    },
  },
  {
    id: 'add-icon',
    title: messages.addIconTitle,
    description: messages.addIconDescription,
    status: 'suggestion',
    shouldShow: (context: NagContext) => !context.project.icon_url,
    link: {
      path: 'settings',
      title: messages.settingsTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'feature-gallery-image',
    title: messages.featureGalleryImageTitle,
    description: messages.featureGalleryImageDescription,
    status: 'suggestion',
    shouldShow: (context: NagContext) => {
      const featuredGalleryImage = context.project.gallery?.find((img) => img.featured)
      return context.project?.gallery?.length === 0 || !featuredGalleryImage
    },
    link: {
      path: 'gallery',
      title: messages.galleryTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-gallery',
    },
  },
  {
    id: 'select-tags',
    title: messages.selectTagsTitle,
    description: messages.selectTagsDescription,
    status: 'suggestion',
    shouldShow: (context: NagContext) =>
      context.project.versions.length > 0 && context.project.categories.length < 1,
    link: {
      path: 'settings/tags',
      title: messages.settingsTagsTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-tags',
    },
  },
  {
    id: 'add-links',
    title: messages.addLinksTitle,
    description: messages.addLinksDescription,
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
      title: messages.settingsLinksTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-links',
    },
  },
  {
    id: 'select-environments',
    title: messages.selectEnvironmentsTitle,
    description: (context: NagContext) => {
      const { formatMessage } = useVIntl()

      return formatMessage(messages.selectEnvironmentsDescription, {
        projectType: formatProjectType(context.project.project_type).toLowerCase(),
      })
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
      title: messages.settingsEnvironmentsTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'select-license',
    title: messages.selectLicenseTitle,
    description: (context: NagContext) => {
      const { formatMessage } = useVIntl()

      return formatMessage(messages.selectLicenseDescription, {
        projectType: formatProjectType(context.project.project_type).toLowerCase(),
      })
    },
    status: 'required',
    shouldShow: (context: NagContext) => context.project.license.id === 'LicenseRef-Unknown',
    link: {
      path: 'settings/license',
      title: messages.settingsLicenseTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-license',
    },
  },
]
