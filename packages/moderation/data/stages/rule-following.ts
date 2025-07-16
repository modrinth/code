import type { Stage } from '../../types/stage'
import type { ButtonAction } from '../../types/actions'
import { ListBulletedIcon } from '@modrinth/assets'

const ruleFollowing: Stage = {
  title: 'Does this project break our content rules?',
  id: 'rule-following',
  icon: ListBulletedIcon,
  guidance_url: 'https://modrinth.com/legal/rules',
  navigate: '/',
  actions: [
    {
      id: 'rule_breaking_yes',
      type: 'button',
      label: 'Yes',
      weight: 10,
      suggestedStatus: 'rejected',
      severity: 'critical',
      message: async () => (await import('../messages/rule-breaking.md?raw')).default,
      relevantExtraInput: [
        {
          label: 'Please explain to the user how it infringes on our content rules.',
          variable: 'MESSAGE',
          required: true,
          large: true,
        },
      ],
    } as ButtonAction,
  ],
}

export default ruleFollowing
