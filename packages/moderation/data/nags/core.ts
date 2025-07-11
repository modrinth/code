import type { Nag, NagContext } from '../../types/nags'
import { formatProjectType } from '@modrinth/utils'

export const coreNags: Nag[] = [
  {
    id: 'upload-version',
    title: 'Upload a version',
    description: () => 'At least one version is required for a project to be submitted for review.',
    status: 'required',
    shouldShow: (context: NagContext) => context.versions.length < 1,
    link: {
      path: 'versions',
      title: 'Visit versions page',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-versions',
    },
  },
  {
    id: 'add-description',
    title: 'Add a description',
    description: () =>
      "A description that clearly describes the project's purpose and function is required.",
    status: 'required',
    shouldShow: (context: NagContext) =>
      context.project.body === '' || context.project.body.startsWith('# Placeholder description'),
    link: {
      path: 'settings/description',
      title: 'Visit description settings',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
    },
  },
  {
    id: 'add-icon',
    title: 'Add an icon',
    description: () =>
      'Your project should have a nice-looking icon to uniquely identify your project at a glance.',
    status: 'suggestion',
    shouldShow: (context: NagContext) => !context.project.icon_url,
    link: {
      path: 'settings',
      title: 'Visit general settings',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'feature-gallery-image',
    title: 'Feature a gallery image',
    description: () => 'Featured gallery images may be the first impression of many users.',
    status: 'suggestion',
    shouldShow: (context: NagContext) => {
      const featuredGalleryImage = context.project.gallery?.find((img) => img.featured)
      return context.project?.gallery?.length === 0 || !featuredGalleryImage
    },
    link: {
      path: 'gallery',
      title: 'Visit gallery page',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-gallery',
    },
  },
  {
    id: 'select-tags',
    title: 'Select tags',
    description: () => 'Select all tags that apply to your project.',
    status: 'suggestion',
    shouldShow: (context: NagContext) =>
      context.project.versions.length > 0 && context.project.categories.length < 1,
    link: {
      path: 'settings/tags',
      title: 'Visit tag settings',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-tags',
    },
  },
  {
    id: 'add-links',
    title: 'Add external links',
    description: () =>
      'Add any relevant links targeted outside of Modrinth, such as sources, issues, or a Discord invite.',
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
      title: 'Visit links settings',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-links',
    },
  },
  {
    id: 'select-environments',
    title: 'Select supported environments',
    description: (context: NagContext) =>
      `Select if the ${formatProjectType(context.project.project_type).toLowerCase()} functions on the client-side and/or server-side.`,
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
      title: 'Visit general settings',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'select-license',
    title: 'Select license',
    description: (context: NagContext) =>
      `Select the license your ${formatProjectType(context.project.project_type).toLowerCase()} is distributed under.`,
    status: 'required',
    shouldShow: (context: NagContext) => context.project.license.id === 'LicenseRef-Unknown',
    link: {
      path: 'settings/license',
      title: 'Visit license settings',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-license',
    },
  },
]
