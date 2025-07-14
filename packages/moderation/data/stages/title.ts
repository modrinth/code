import { BookOpenIcon } from '@modrinth/assets'
import type { Stage } from '../../types/stage'

const titleStage: Stage = {
  title: 'Is this title free of useless information?',
  id: 'title',
  text: async () => '**Title:** %PROJECT_TITLE%',
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
      weight: 110,
      suggestedStatus: 'flagged',
      severity: 'medium',
      message: async () => (await import('../messages/title/similarities.md?raw')).default,
      enablesActions: [
        {
          id: 'title_similarities_select_modpack',
          type: 'multi-select-chips',
          label: 'Similarities additional info',
          shouldShow: (project) => project.project_type === 'modpack',
          options: [
            {
              label: 'Modpack named after mod',
              weight: 111,
              message: async () =>
                (await import('../messages/title/similarities-modpack.md?raw')).default,
            },
            {
              label: 'Forked project',
              weight: 112,
              message: async () =>
                (await import('../messages/title/similarities-fork.md?raw')).default,
            },
          ],
        },
        {
          id: 'title_similarities_select_not_modpack',
          type: 'multi-select-chips',
          label: 'Similarities additional info',
          shouldShow: (project) => project.project_type !== 'modpack',
          options: [
            {
              label: 'Forked project',
              weight: 112,
              message: async () =>
                (await import('../messages/title/similarities-fork.md?raw')).default,
            },
          ],
        },
      ],
    },
  ],
}

export default titleStage
