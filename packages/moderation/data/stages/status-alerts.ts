import type { Stage } from '../../types/stage'
import type { ButtonAction } from '../../types/actions'
import { TriangleAlertIcon } from '@modrinth/assets'

const statusAlerts: Stage = {
  title: `Is anything else affecting this project's status?`,
  id: 'Status Alerts',
  icon: TriangleAlertIcon,
  guidance_url:
    'https://www.notion.so/Project-Modification-Guidelines-22e5ee711bf080628416f0471ba6af02',
  navigate: '/moderation',
  actions: [
    {
      id: 'status_corrections_applied',
      type: 'button',
      label: 'Corrections applied',
      weight: 1000,
      suggestedStatus: 'approved',
      disablesActions: ['status_private_use'],
      message: async () => (await import('../messages/fixed.md?raw')).default,
      /*       conditionalMessages: [
        {
          // title corrected
          conditions: {
            requiredActions: ['this_is_not_a_real_id'],
          },
          message: async () =>
            (await import('../messages/fixed.md?raw')).default +
            (await import('../messages/corrections/title.md?raw')).default,
          fallbackMessage: async () => (await import('../messages/fixed.md?raw')).default,
        } as ConditionalMessage,
        {
          // optimization tag removed
          conditions: {
            requiredActions: ['corrections_applied', 'categories_optimization_misused'],
          },
          message: async () =>
            (await import('../messages/fixed.md?raw')).default +
            (await import('../messages/corrections/optimization_tag.md?raw')).default,
        } as ConditionalMessage,
      ], */
    } as ButtonAction,
    {
      id: 'status_private_use',
      type: 'button',
      label: 'Private use',
      weight: 1000,
      suggestedStatus: 'flagged',
      disablesActions: ['status_corrections_applied'],
      message: async () => (await import('../messages/private.md?raw')).default,
    } as ButtonAction,
  ],
}

export default statusAlerts
