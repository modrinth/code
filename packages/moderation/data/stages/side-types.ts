import type { Stage } from '../../types/stage'
import type { ButtonAction } from '../../types/actions'
import { GlobeIcon } from '@modrinth/assets'

const sideTypes: Stage = {
  title: "Is the project's environment information accurate?",
  id: 'environment',
  icon: GlobeIcon,
  guidance_url: 'https://modrinth.com/legal/rules#miscellaneous',
  navigate: '/settings#side-types',
  actions: [
    {
      id: 'side_types_inaccurate_modpack',
      type: 'button',
      label: 'Inaccurate (modpack)',
      weight: 10,
      suggestedStatus: 'flagged',
      severity: 'low',
      shouldShow: (project) => project.project_type === 'modpack',
      message: async () =>
        (await import('../messages/side-types/inaccurate-modpack.md?raw')).default,
    } as ButtonAction,
    {
      id: 'side_types_inaccurate_mod',
      type: 'button',
      label: 'Inaccurate (mod)',
      weight: 10,
      suggestedStatus: 'flagged',
      severity: 'low',
      shouldShow: (project) => project.project_type === 'mod',
      message: async () => (await import('../messages/side-types/inaccurate-mod.md?raw')).default,
    } as ButtonAction,
  ],
}

export default sideTypes
