import type { Stage } from '../../types/stage'
import type { ButtonAction } from '../../types/actions'

const categories: Stage = {
  title: "Are the project's tags/categories accurate?",
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
      message: async () => await import('../messages/categories/inaccurate.md?raw'),
    } as ButtonAction,
  ],
}

export default categories
