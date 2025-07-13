import type { Project } from '@modrinth/utils'
import type { Nag, NagContext } from '../../types/nags'

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
    title: 'Too many tags selected',
    description: (context: NagContext) => {
      const tagCount =
        context.project.categories.length + context.project.additional_categories?.length || 0
      return `You've selected ${tagCount} tags. Consider reducing to 5 or fewer to keep your project focused and easier to discover.`
    },
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const tagCount =
        context.project.categories.length + context.project.additional_categories?.length || 0
      return tagCount > 5
    },
    link: {
      path: 'settings/tags',
      title: 'Edit tags',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-tags',
    },
  },
  {
    id: 'multiple-resolution-tags',
    title: 'Multiple resolution tags selected',
    description: (context: NagContext) => {
      const resolutionTags = context.project.categories.filter((tag: string) =>
        ['16x', '32x', '48x', '64x', '128x', '256x', '512x', '1024x'].includes(tag),
      )
      return `You've selected ${resolutionTags.length} resolution tags (${resolutionTags.join(', ')}). Resource packs should typically only have one resolution tag that matches their primary resolution.`
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
      title: 'Edit tags',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-tags',
    },
  },
  {
    id: 'all-tags-selected',
    title: 'All tags selected',
    description: (context: NagContext) => {
      const categoriesForProjectType = getCategories(
        context.project as Project & { actualProjectType: string },
        context.tags,
      )
      const totalAvailableTags = categoriesForProjectType.length
      return `You've selected all ${totalAvailableTags} available tags. This defeats the purpose of tags, which are meant to help users find relevant projects. Please select only the tags that truly apply to your project.`
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
      title: 'Edit tags',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-tags',
    },
  },
]
