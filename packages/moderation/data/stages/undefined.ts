import { BookOpenIcon } from '@modrinth/assets'
import type { Stage } from '../../types/stage'

const undefinedStage: Stage = {
  title: 'This project is undefined!',
  id: 'undefined',
  icon: BookOpenIcon,
  guidance_url: 'https://modrinth.com/legal/rules#miscellaneous',
  navigate: '/versions',
  shouldShow: (project) => project.versions.length === 0,
  actions: [
    {
      id: 'undefined_no_versions',
      type: 'button',
      label: 'No Versions',
      weight: 100,
      suggestedStatus: 'rejected',
      message: async () => (await import('../messages/undefined/no_versions.md?raw')).default,
    },
  ],
}

export default undefinedStage
