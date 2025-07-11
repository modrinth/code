import { BookTextIcon } from '@modrinth/assets'
import type { Stage } from '../../types/stage'

const licenseStage: Stage = {
  title: 'Is this license and link valid?',
  text: async (project) =>
    project.license.url ? '**License Link:** `%PROJECT_LICENSE_URL%`' : 'No Link',
  id: 'license',
  icon: BookTextIcon,
  guidance_url: 'https://modrinth.com/legal/rules#miscellaneous',
  navigate: '/settings/license',

  actions: [
    {
      id: 'license_invalid_link',
      type: 'button',
      label: 'Invalid License Link',
      weight: 100,
      suggestedStatus: 'flagged',
      severity: 'medium',
      shouldShow: (project) => (project.license.url ? true : false),
      message: async () => (await import('../messages/license/invalid_link.md?raw')).default,

      enablesActions: [
        {
          id: 'license_invalid_link-custom_license',
          type: 'button',
          label: 'Custom License',
          weight: 100,
          suggestedStatus: 'flagged',
          severity: 'medium',
          message: async () =>
            (await import('../messages/license/invalid_link-custom_license.md?raw')).default,
        },
      ],
    },
  ],
}

export default licenseStage
