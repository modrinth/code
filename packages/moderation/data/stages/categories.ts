import type { Stage } from '../../types/stage'
import type { ButtonAction } from '../../types/actions'
import { TagsIcon } from '@modrinth/assets'

const categories: Stage = {
  title: "Are the project's tags accurate?",
  id: 'tags',
  icon: TagsIcon,
  guidance_url: 'https://modrinth.com/legal/rules#miscellaneous',
  navigate: '/settings/tags',
  actions: [
    {
      id: 'categories_inaccurate',
      type: 'button',
      label: 'Inaccurate',
      weight: 10,
      suggestedStatus: 'flagged',
      severity: 'low',
      message: async () => (await import('../messages/categories/inaccurate.md?raw')).default,
      disablesActions: ['categories_optimization_misused', 'categories_resolutions_misused'],
    } as ButtonAction,
    {
      id: 'categories_optimization_misused',
      type: 'button',
      label: 'Optimization',
      weight: 10,
      suggestedStatus: 'flagged',
      severity: 'low',
      message: async () =>
        (await import('../messages/categories/inaccurate.md?raw')).default +
        (await import('../messages/categories/optimization_misused.md?raw')).default,
      disablesActions: ['categories_inaccurate', 'categories_resolutions_misused'],
    } as ButtonAction,
    {
      id: 'categories_resolutions_misused',
      type: 'button',
      label: 'Resolutions',
      weight: 10,
      suggestedStatus: 'flagged',
      severity: 'low',
      shouldShow: (project) => project.project_type === 'resourcepack',
      message: async () =>
        (await import('../messages/categories/inaccurate.md?raw')).default +
        (await import('../messages/categories/resolutions_misused.md?raw')).default,
      disablesActions: ['categories_inaccurate', 'categories_optimization_misused'],
    },
  ],
}

export default categories
