import type { Stage } from '../../types/stage'
import type { ButtonAction } from '../../types/actions'
import { AlignLeftIcon } from '@modrinth/assets'

const summary: Stage = {
  title: "Is the project's summary sufficient?",
  id: 'summary',
  icon: AlignLeftIcon,
  guidance_url: 'https://modrinth.com/legal/rules#miscellaneous',
  actions: [
    {
      id: 'summary_insufficient',
      type: 'button',
      label: 'Insufficient',
      weight: 10,
      suggestedStatus: 'flagged',
      severity: 'low',
      message: async () => (await import('../messages/summary/insufficient.md?raw')).default,
    } as ButtonAction,
    {
      id: 'summary_repeat_title',
      type: 'button',
      label: 'Repeat of title',
      weight: 10,
      suggestedStatus: 'flagged',
      severity: 'low',
      message: async () => (await import('../messages/summary/repeat-title.md?raw')).default,
    } as ButtonAction,
    {
      id: 'summary_formatting',
      type: 'button',
      label: 'Formatting',
      weight: 10,
      suggestedStatus: 'flagged',
      severity: 'low',
      message: async () => (await import('../messages/summary/formatting.md?raw')).default,
    } as ButtonAction,
    {
      id: 'summary_non_english',
      type: 'button',
      label: 'Non-english',
      weight: 10,
      suggestedStatus: 'flagged',
      severity: 'medium',
      message: async () => (await import('../messages/summary/non-english.md?raw')).default,
    } as ButtonAction,
  ],
}

export default summary
