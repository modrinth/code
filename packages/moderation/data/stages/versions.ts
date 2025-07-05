import type { Stage } from '../../types/stage'
import type { ButtonAction } from '../../types/actions'
import { VersionIcon } from '@modrinth/assets'

const versions: Stage = {
  title: "Are these project's files correct?",
  id: 'versions',
  icon: VersionIcon,
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
      message: async () =>
        (await import('../messages/versions/incorrect-additional-files.md?raw')).default,
    } as ButtonAction,
    {
      id: 'versions_invalid_modpacks',
      type: 'button',
      label: 'Invalid file type (modpacks)',
      weight: 10,
      suggestedStatus: 'rejected',
      severity: 'medium',
      shouldShow: (project) => project.project_type === 'modpack',
      message: async () => (await import('../messages/versions/invalid-modpacks.md?raw')).default,
    } as ButtonAction,
    {
      id: 'versions_invalid_resourcepacks',
      type: 'button',
      label: 'Invalid file type (resourcepacks)',
      weight: 10,
      suggestedStatus: 'rejected',
      severity: 'medium',
      shouldShow: (project) => project.project_type === 'resourcepack',
      message: async () =>
        (await import('../messages/versions/invalid-resourcepacks.md?raw')).default,
    } as ButtonAction,
  ],
}

export default versions
