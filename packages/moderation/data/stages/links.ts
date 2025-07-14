import type { Stage } from '../../types/stage'
import type { ButtonAction } from '../../types/actions'
import { LinkIcon } from '@modrinth/assets'

const links: Stage = {
  title: "Are the project's links accurate and accessible?",
  id: 'links',
  icon: LinkIcon,
  guidance_url: 'https://modrinth.com/legal/rules#miscellaneous',
  navigate: '/settings/links',
  shouldShow: (project) =>
    Boolean(
      project.issues_url ||
        project.source_url ||
        project.wiki_url ||
        project.discord_url ||
        project.donation_urls.length > 0,
    ),
  text: async (project) => {
    let text = ''
    if (project.issues_url)
      text += (await import('../messages/checklist-text/links/issues.md?raw')).default
    if (project.source_url)
      text += (await import('../messages/checklist-text/links/source.md?raw')).default
    if (project.wiki_url)
      text += (await import('../messages/checklist-text/links/wiki.md?raw')).default
    if (project.discord_url)
      text += (await import('../messages/checklist-text/links/discord.md?raw')).default

    if (project.donation_urls.length > 0) {
      text += (await import('../messages/checklist-text/links/donation/donations.md?raw')).default

      for (const donation of project.donation_urls) {
        text += (await import(`../messages/checklist-text/links/donation/donation.md?raw`)).default
          .replace('{URL}', donation.url)
          .replace('{PLATFORM}', donation.platform)
      }
    }

    if (text !== '') return text
    return 'No links provided.'
  },
  actions: [
    {
      id: 'links_misused',
      type: 'button',
      label: 'Links are misused',
      weight: 500,
      suggestedStatus: 'flagged',
      severity: 'low',
      message: async () => (await import('../messages/links/misused.md?raw')).default,
      relevantExtraInput: [
        {
          label: 'What links are misused?',
          variable: 'MISUSED_LINKS',
          required: false,
        },
      ],
    } as ButtonAction,
    {
      id: 'links_unaccessible',
      type: 'button',
      label: 'Links or unaccessible',
      weight: 510,
      suggestedStatus: 'flagged',
      // Theoretically a conditional could go here to prevent overlap of misuse and unaccessible messages repeating while still allowing for a multi-select in each.
      // if links_misused was selected, send nothing.
      message: async () => (await import('../messages/links/not_accessible.md?raw')).default,
      enablesActions: [
        {
          id: 'links_unaccessible_options',
          type: 'multi-select-chips',
          label: 'Warn of unaccessible link?',
          options: [
            {
              label: 'Source',
              weight: 511,
              shouldShow: (project) => Boolean(project.source_url),
              message: async () =>
                (await import('../messages/links/not_accessible-source.md?raw')).default,
            },
            {
              label: 'Discord',
              weight: 512,
              shouldShow: (project) => Boolean(project.discord_url),
              message: async () =>
                (await import('../messages/links/not_accessible-discord.md?raw')).default,
            },
          ],
        },
      ],
    } as ButtonAction,
  ],
}

export default links
