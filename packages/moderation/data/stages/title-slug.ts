import { BookOpenIcon } from '@modrinth/assets'
import type { Stage } from '../../types/stage'

const titleSlug: Stage = {
  title: 'Are the Name and URL accurate and appropriate?',
  id: 'title-&-slug',
  text: async () => (await import('../messages/checklist-text/title-slug.md?raw')).default,
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
          id: 'title_similarities_options',
          type: 'multi-select-chips',
          label: 'Similarities additional info',
          options: [
            {
              label: 'Modpack named after mod',
              weight: 111,
              shouldShow: (project) => project.project_type === 'modpack',
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
      ],
    },
    {
      id: 'slug_misused_options',
      type: 'multi-select-chips',
      label: 'Slug issues?',
      suggestedStatus: 'rejected',
      severity: 'low',
      options: [
        {
          label: 'Misused',
          weight: 200,
          message: async () => (await import('../messages/slug/misused.md?raw')).default,
        },
      ],
    },
  ],
}

export default titleSlug
