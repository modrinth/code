import type { Nag, NagContext } from '../../types/nags'
import { formatProjectType } from '@modrinth/utils'

export const commonLinkDomains = {
  source: ['github.com', 'gitlab.com', 'bitbucket.org', 'codeberg.org', 'git.sr.ht'],
  issues: ['github.com', 'gitlab.com', 'bitbucket.org', 'codeberg.org'],
  discord: ['discord.gg', 'discord.com'],
}

export function isCommonUrl(url: string | undefined, commonDomains: string[]): boolean {
  if (!url) return false
  try {
    const domain = new URL(url).hostname.toLowerCase()
    console.log(domain)
    return commonDomains.some((allowed) => domain.includes(allowed))
  } catch {
    return true
  }
}

export const linksNags: Nag[] = [
  {
    id: 'verify-external-links',
    title: 'Verify external links',
    description: () =>
      `Some of your external links may be using domains that aren't recognized as common for their link type.`,
    status: 'warning',
    shouldShow: (context: NagContext) => {
      return (
        !isCommonUrl(context.project.source_url, commonLinkDomains.source) ||
        !isCommonUrl(context.project.issues_url, commonLinkDomains.issues) ||
        !isCommonUrl(context.project.discord_url, commonLinkDomains.discord)
      )
    },
    link: {
      path: 'settings/links',
      title: 'Visit links settings',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-links',
    },
  },
  {
    id: 'gpl-license-source-required',
    title: 'GPL license requires source',
    description: (context: NagContext) =>
      `Your ${formatProjectType(context.project.project_type).toLowerCase()} uses a GPL license which requires source code to be available. Please provide a source code link or consider using a different license.`,
    status: 'required',
    shouldShow: (context: NagContext) => {
      const gplLicenses = [
        'GPL-2.0',
        'GPL-2.0+',
        'GPL-2.0-only',
        'GPL-2.0-or-later',
        'GPL-3.0',
        'GPL-3.0+',
        'GPL-3.0-only',
        'GPL-3.0-or-later',
        'LGPL-2.1',
        'LGPL-2.1+',
        'LGPL-2.1-only',
        'LGPL-2.1-or-later',
        'LGPL-3.0',
        'LGPL-3.0+',
        'LGPL-3.0-only',
        'LGPL-3.0-or-later',
        'AGPL-3.0',
        'AGPL-3.0+',
        'AGPL-3.0-only',
        'AGPL-3.0-or-later',
      ]

      const isGplLicense = gplLicenses.includes(context.project.license.id)
      const hasSourceUrl = !!context.project.source_url

      return isGplLicense && !hasSourceUrl
    },
    link: {
      path: 'settings/links',
      title: 'Visit links settings',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-links',
    },
  },
]
