import type { Stage } from '../../types/stage'
import type { ButtonAction } from '../../types/actions'
import { TriangleAlertIcon } from '@modrinth/assets'

const statusAlerts: Stage = {
  title: `Is anything else affecting this project's status?`,
  id: 'status-alerts',
  icon: TriangleAlertIcon,
  text: async () => '**Applying for:** `%PROJECT_REQUESTED_STATUS%`',
  guidance_url:
    'https://www.notion.so/Project-Modification-Guidelines-22e5ee711bf080628416f0471ba6af02',
  navigate: '/moderation',
  actions: [
    {
      id: 'status_corrections_applied',
      type: 'button',
      label: 'Corrections applied',
      weight: 999999,
      suggestedStatus: 'approved',
      disablesActions: ['status_private_use'],
      message: async () => (await import('../messages/fixed.md?raw')).default,
    } as ButtonAction,
    {
      id: 'status_private_use',
      type: 'button',
      label: 'Private use',
      weight: 999999,
      suggestedStatus: 'flagged',
      disablesActions: ['status_corrections_applied'],
      message: async () => (await import('../messages/private.md?raw')).default,
    } as ButtonAction,
  ],
}

export default statusAlerts
