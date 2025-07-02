import type { Stage } from '../../types/stage'
import type { ButtonAction } from '../../types/actions'

const ruleFollowing: Stage = {
  title: 'Does this project follow our content rules?',
  guidance_url: 'https://modrinth.com/legal/rules',
  navigate: '/',
  actions: [
    {
      id: 'rule_following_no',
      type: 'button',
      label: 'No',
      weight: 10,
      suggestedStatus: 'rejected',
      severity: 'critical',
      message: async () => await import('../messages/rule-breaking.md?raw'),
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
