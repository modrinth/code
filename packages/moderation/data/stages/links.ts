import type { Stage } from '../../types/stage'
import type { ButtonAction } from '../../types/actions'

const links: Stage = {
  title: "Are the project's links accessible and not misleading?",
  guidance_url: 'https://modrinth.com/legal/rules#miscellaneous',
  navigate: '/settings/links',
  actions: [
    {
      id: 'links_misused',
      type: 'button',
      label: 'Links are misused',
      weight: 10,
      suggestedStatus: 'flagged',
      severity: 'low',
      message: async () => await import('../messages/links/misused.md?raw'),
    } as ButtonAction,
    {
      id: 'links_not_accessible_source',
      type: 'button',
      label: 'Not accessible (source)',
      weight: 10,
      suggestedStatus: 'flagged',
      severity: 'low',
      message: async () => await import('../messages/links/not-accessible-source.md?raw'),
    } as ButtonAction,
    {
      id: 'links_not_accessible_other',
      type: 'button',
      label: 'Not accessible (other)',
      weight: 10,
      suggestedStatus: 'flagged',
      severity: 'low',
      message: async () => await import('../messages/links/not-accessible-other.md?raw'),
      relevantExtraInput: [
        {
          label: 'Please specify the link type that is inaccessible.',
          variable: 'LINK',
          required: true,
        },
      ],
    } as ButtonAction,
  ],
}

export default links
