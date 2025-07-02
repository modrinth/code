import type { Stage } from '../types/stage'

export default [
  {
    title: 'Mandatory Cat Picture',
    guidance_url: 'https://notion.so/modrinth/moderation-guidelines#mandatory-cat-picture',
    actions: [
      {
        type: 'button',
        weight: 999,
        label: 'No Cat Picture',
        message: async () => await import('../messages/example.md?raw'),
      },
    ],
  },
] as ReadonlyArray<Stage>
