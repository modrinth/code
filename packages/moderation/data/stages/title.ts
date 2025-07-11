import { BookOpenIcon } from '@modrinth/assets'
import type { Stage } from '../../types/stage'

const titleStage: Stage = {
  title: 'Is this title free of useless information?',
  text: async () => '**Title:** `%PROJECT_TITLE%`',
  id: 'title',
  icon: BookOpenIcon,
  guidance_url: 'https://modrinth.com/legal/rules#miscellaneous',
  actions: [
    {
      id: 'title_useless_info',
      type: 'button',
      label: 'Contains useless info',
      weight: 100,
      suggestedStatus: 'flagged',
      severity: 'low',
      message: async () => (await import('../messages/title/useless-info.md?raw')).default,
    },
    {
      id: 'title_minecraft_branding',
      type: 'button',
      label: 'Minecraft title',
      weight: 100,
      suggestedStatus: 'flagged',
      severity: 'medium',
      message: async () => (await import('../messages/title/minecraft-branding.md?raw')).default,
    },
    {
      id: 'title_similarities',
      type: 'button',
      label: 'Title similarities',
      weight: 100,
      suggestedStatus: 'flagged',
      severity: 'medium',
      message: async () => (await import('../messages/title/similarities.md?raw')).default,
    },
  ],
}

export default titleStage
