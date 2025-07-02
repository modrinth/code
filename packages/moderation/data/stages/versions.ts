import type { Stage } from '../../types/stage'
import type { ButtonAction } from '../../types/actions'

const versions: Stage = {
  title: "Are these project's files correct?",
  guidance_url: 'https://modrinth.com/legal/rules#miscellaneous',
  navigate: '/versions',
  actions: [
    {
      id: 'versions_incorrect_additional',
      type: 'button',
      label: 'Incorrect additional files',
      weight: 10,
      suggestedStatus: 'flagged',
      severity: 'medium',
      message: async () => await import('../messages/versions/incorrect-additional-files.md?raw'),
    } as ButtonAction,
    {
      id: 'versions_invalid_modpacks',
      type: 'button',
      label: 'Invalid file type (modpacks)',
      weight: 10,
      suggestedStatus: 'rejected',
      severity: 'medium',
      message: async () => await import('../messages/versions/invalid-modpacks.md?raw'),
    } as ButtonAction,
    {
      id: 'versions_invalid_resourcepacks',
      type: 'button',
      label: 'Invalid file type (resourcepacks)',
      weight: 10,
      suggestedStatus: 'rejected',
      severity: 'medium',
      message: async () => await import('../messages/versions/invalid-resourcepacks.md?raw'),
    } as ButtonAction,
  ],
}

export default versions
