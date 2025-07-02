import type { Stage } from '../../types/stage'
import type { ButtonAction } from '../../types/actions'

const gallery: Stage = {
  title: "Are this project's gallery images sufficient?",
  guidance_url: 'https://modrinth.com/legal/rules#general-expectations',
  navigate: '/gallery',
  actions: [
    {
      id: 'gallery_insufficient',
      type: 'button',
      label: 'Insufficient',
      weight: 10,
      suggestedStatus: 'flagged',
      severity: 'low',
      message: async () => await import('../messages/gallery/insufficient.md?raw'),
    } as ButtonAction,
    {
      id: 'gallery_not_relevant',
      type: 'button',
      label: 'Not relevant',
      weight: 10,
      suggestedStatus: 'flagged',
      severity: 'low',
      message: async () => await import('../messages/gallery/not-relevant.md?raw'),
    } as ButtonAction,
  ],
}

export default gallery
