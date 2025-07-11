import type { Stage } from '../../types/stage'
import type { ButtonAction } from '../../types/actions'
import { ListBulletedIcon } from '@modrinth/assets'

const ruleFollowing: Stage = {
  title: 'Does this project follow our content rules?',
  id: 'rule-following',
  icon: ListBulletedIcon,
  guidance_url:
    'https://www.notion.so/Creator-Communication-Guide-1b65ee711bf080ec9337e3ccdded146c',
  navigate: '/',
  actions: [
    {
      id: 'rule_following_no',
      type: 'button',
      label: 'No',
      weight: 10,
      suggestedStatus: 'rejected',
      severity: 'critical',
      message: async () => (await import('../messages/rule-breaking.md?raw')).default,
      relevantExtraInput: [
        {
          label: 'Please explain to the user how it infringes on our content rules.',
          variable: 'MESSAGE',
          required: true,
        },
      ],
    } as ButtonAction,
  ],
}

export default ruleFollowing
