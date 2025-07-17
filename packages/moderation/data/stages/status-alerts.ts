import type { Stage } from '../../types/stage'
import type { ButtonAction } from '../../types/actions'
import { TriangleAlertIcon } from '@modrinth/assets'

const statusAlerts: Stage = {
  title: `Is anything else affecting this project's status?`,
  id: 'status-alerts',
  icon: TriangleAlertIcon,
  text: async () => (await import('../messages/checklist-text/status-alerts/text.md?raw')).default,
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
      disablesActions: ['status_private_use', 'status_account_issues'],
      message: async () => (await import('../messages/status-alerts/fixed.md?raw')).default,
    } as ButtonAction,
    {
      id: 'status_private_use',
      type: 'button',
      label: 'Private use',
      weight: 999999,
      suggestedStatus: 'flagged',
      disablesActions: ['status_corrections_applied', 'status_account_issues'],
      message: async () => (await import('../messages/status-alerts/private.md?raw')).default,
    } as ButtonAction,
    {
      id: 'status_account_issues',
      type: 'button',
      label: 'Account issues',
      weight: 999999,
      suggestedStatus: 'rejected',
      disablesActions: ['status_corrections_applied', 'status_private_use'],
      message: async () =>
        (await import('../messages/status-alerts/account_issues.md?raw')).default,
    } as ButtonAction,
    {
      id: 'status_automod_confusion',
      type: 'button',
      label: `Automod confusion`,
      weight: 999999,
      message: async () =>
        (await import('../messages/status-alerts/automod_confusion.md?raw')).default,
    } as ButtonAction,
  ],
}

export default statusAlerts
