import { BookTextIcon } from '@modrinth/assets'
import type { Stage } from '../../types/stage'

const licensesNotRequiringSource: string[] = [
  'LicenseRef-All-Rights-Reserved',
  'Apache-2.0',
  'BSD-2-Clause',
  'BSD-3-Clause',
  'CC0-1.0',
  'CC-BY-4.0',
  'CC-BY-SA-4.0',
  'CC-BY-NC-4.0',
  'CC-BY-NC-SA-4.0',
  'CC-BY-ND-4.0',
  'CC-BY-NC-ND-4.0',
  'ISC',
  'MIT',
  'Zlib',
]

const licenseStage: Stage = {
  title: 'Is this license and link valid?',
  text: async (project) => {
    let text = ''
    text += (await import('../messages/checklist-text/license/id.md?raw')).default
    if (project.license.url)
      text += (await import('../messages/checklist-text/license/link.md?raw')).default

    return text
  },
  id: 'license',
  icon: BookTextIcon,
  guidance_url: 'https://modrinth.com/legal/rules#miscellaneous',
  navigate: '/settings/license',
  actions: [
    {
      id: 'license_invalid_link',
      type: 'button',
      label: 'Invalid Link',
      weight: 600,
      suggestedStatus: 'flagged',
      severity: 'medium',
      shouldShow: (project) => Boolean(project.license.url),
      message: async () => (await import('../messages/license/invalid_link.md?raw')).default,
      enablesActions: [
        {
          id: 'license_invalid_link-custom_license',
          type: 'toggle',
          label: 'Invalid Link: Custom License',
          weight: 601,
          suggestedStatus: 'flagged',
          severity: 'medium',
          message: async () =>
            (await import('../messages/license/invalid_link-custom_license.md?raw')).default,
        },
      ],
    },
    //    {
    //      id: 'license_no_source',
    //      type: 'conditional-button',
    //      label: 'No Source',
    //      fallbackWeight: 602,
    //      suggestedStatus: 'rejected',
    //      severity: 'medium',
    //      fallbackMessage: async () => (await import('../messages/license/no_source.md?raw')).default,
    //      messageVariants: [
    //        {
    //          conditions: {
    //            requiredActions: ['reupload_unclear_fork'],
    //          },
    //          weight: 602,
    //          message: async () => (await import('../messages/license/no_source-fork.md?raw')).default,
    //        },
    //      ],
    //    },
    {
      id: 'license_no_source',
      type: 'button',
      label: 'No Source',
      weight: 602,
      suggestedStatus: 'rejected',
      severity: 'medium',
      shouldShow: (project) => !licensesNotRequiringSource.includes(project.license.id),
      message: async () => (await import('../messages/license/no_source.md?raw')).default,
      enablesActions: [
        {
          id: 'license_no_source-fork',
          type: 'toggle',
          label: 'No Source: Fork',
          weight: 602,
          suggestedStatus: 'rejected',
          severity: 'high',
          message: async () => (await import('../messages/license/no_source-fork.md?raw')).default,
        },
      ],
    },
  ],
}

export default licenseStage
