import { HashIcon } from '@modrinth/assets'
import type { Stage } from '../../types/stage'

const slugStage: Stage = {
  title: 'Is the slug accurate and appropriate?',
  id: 'slug',
  icon: HashIcon,
  guidance_url: 'https://modrinth.com/legal/rules#miscellaneous',
  navigate: '/settings',
  actions: [
    {
      id: 'slug_misused',
      type: 'button',
      label: 'Misused',
      weight: 100,
      suggestedStatus: 'flagged',
      severity: 'low',
      message: async () => (await import('../messages/slug/misused.md?raw')).default,
    },
  ],
}

export default slugStage
