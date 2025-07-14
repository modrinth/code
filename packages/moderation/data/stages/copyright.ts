import type { Stage } from '../../types/stage'
import type { ButtonAction } from '../../types/actions'
import { CopyrightIcon } from '@modrinth/assets'

const copyright: Stage = {
  title: 'Does the author have proper permissions to post this project?',
  id: 'copyright',
  icon: CopyrightIcon,
  guidance_url: 'https://modrinth.com/legal/rules',
  actions: [
    {
      id: 'copyright_reupload',
      type: 'button',
      label: 'Re-upload',
      weight: 1100,
      suggestedStatus: 'rejected',
      severity: 'high',
      message: async () => (await import('../messages/copyright/reupload.md?raw')).default,
      relevantExtraInput: [
        {
          label: 'What is the title of the original project?',
          variable: 'ORIGINAL_PROJECT',
          required: true,
          suggestions: ['Vanilla Tweaks'],
        },
        {
          label: 'What is the author of the original project?',
          variable: 'ORIGINAL_AUTHOR',
          required: true,
          suggestions: ['Vanilla Tweaks Team'],
        },
      ],
    } as ButtonAction,
    {
      id: 'copyright_insifficient_fork',
      type: 'button',
      label: 'Insufficient Fork',
      weight: 1101,
      suggestedStatus: 'flagged',
      severity: 'medium',
      message: async () => (await import('../messages/copyright/insufficient_fork.md?raw')).default,
    },
  ],
}

export default copyright
