import type { Project } from '@modrinth/utils'
import type { Nag, NagContext } from '../../types/nags'
import { useVIntl } from '@vintl/vintl'

import messages from './tags.i18n'

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
    title: messages.tooManyTagsTitle,
    description: (context: NagContext) => {
      const { formatMessage } = useVIntl()
      const tagCount =
        context.project.categories.length + (context.project.additional_categories?.length || 0)

      return formatMessage(messages.tooManyTagsDescription, {
        tagCount,
      })
    },
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const tagCount =
        context.project.categories.length + (context.project.additional_categories?.length || 0)
      return tagCount > 5
    },
    link: {
      path: 'settings/tags',
      title: messages.editTagsTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-tags',
    },
  },
  {
    id: 'multiple-resolution-tags',
    title: messages.multipleResolutionTagsTitle,
    description: (context: NagContext) => {
      const { formatMessage } = useVIntl()
      const resolutionTags = context.project.categories.filter((tag: string) =>
        ['16x', '32x', '48x', '64x', '128x', '256x', '512x', '1024x'].includes(tag),
      )

      return formatMessage(messages.multipleResolutionTagsDescription, {
        count: resolutionTags.length,
        tags: resolutionTags.join(', '),
      })
    },
    status: 'warning',
    shouldShow: (context: NagContext) => {
      if (context.project.project_type !== 'resourcepack') return false

      const resolutionTags = context.project.categories.filter((tag: string) =>
        ['16x', '32x', '48x', '64x', '128x', '256x', '512x', '1024x'].includes(tag),
      )
      return resolutionTags.length > 1
    },
    link: {
      path: 'settings/tags',
      title: messages.editTagsTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-tags',
    },
  },
  {
    id: 'all-tags-selected',
    title: messages.allTagsSelectedTitle,
    description: (context: NagContext) => {
      const { formatMessage } = useVIntl()
      const categoriesForProjectType = getCategories(
        context.project as Project & { actualProjectType: string },
        context.tags,
      )
      const totalAvailableTags = categoriesForProjectType.length

      return formatMessage(messages.allTagsSelectedDescription, {
        totalAvailableTags,
      })
    },
    status: 'required',
    shouldShow: (context: NagContext) => {
      const categoriesForProjectType = getCategories(
        context.project as Project & { actualProjectType: string },
        context.tags,
      )
      const totalSelectedTags =
        context.project.categories.length + (context.project.additional_categories?.length || 0)
      return totalSelectedTags === categoriesForProjectType.length
    },
    link: {
      path: 'settings/tags',
      title: messages.editTagsTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-tags',
    },
  },
]
