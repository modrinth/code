import type { Stage } from '../../types/stage'
import type { ButtonAction } from '../../types/actions'

const sideTypes: Stage = {
  title: "Is the project's environment information accurate?",
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
      message: async () => await import('../messages/side-types/inaccurate-modpack.md?raw'),
    } as ButtonAction,
    {
      id: 'side_types_inaccurate_mod',
      type: 'button',
      label: 'Inaccurate (mod)',
      weight: 10,
      suggestedStatus: 'flagged',
      severity: 'low',
      message: async () => await import('../messages/side-types/inaccurate-mod.md?raw'),
    } as ButtonAction,
  ],
}

export default sideTypes
