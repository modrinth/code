import type { Nag, NagContext } from '../../types/nags'
import { formatProjectType } from '@modrinth/utils'

export const commonLinkDomains = {
  source: ['github.com', 'gitlab.com', 'bitbucket.org', 'codeberg.org', 'git.sr.ht'],
  issues: ['github.com', 'gitlab.com', 'bitbucket.org', 'codeberg.org'],
  discord: ['discord.gg', 'discord.com'],
  licenseBlocklist: [
    'youtube.com',
    'youtu.be',
    'modrinth.com',
    'curseforge.com',
    'twitter.com',
    'x.com',
    'discord.gg',
    'discord.com',
    'instagram.com',
    'facebook.com',
    'tiktok.com',
    'reddit.com',
    'twitch.tv',
    'patreon.com',
    'ko-fi.com',
    'paypal.com',
    'buymeacoffee.com',
  ],
}

export function isCommonUrl(url: string | undefined, commonDomains: string[]): boolean {
  if (!url) return false
  try {
    const domain = new URL(url).hostname.toLowerCase()
    return commonDomains.some((allowed) => domain.includes(allowed))
  } catch {
    return false
  }
}

export function isUncommonLicenseUrl(url: string | undefined, domains: string[]): boolean {
  if (!url) return false
  try {
    const domain = new URL(url).hostname.toLowerCase()
    return domains.some((uncommonDomain) => domain.includes(uncommonDomain))
  } catch {
    return false
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
    id: 'invalid-license-url',
    title: 'Invalid license URL',
    description: (context: NagContext) => {
      const licenseUrl = context.project.license.url
      if (!licenseUrl) return 'License URL is invalid.'

      try {
        const domain = new URL(licenseUrl).hostname.toLowerCase()
        return `Your license URL points to ${domain}, which is not appropriate for license information. License URLs should link to the actual license text or legal documentation, not social media, gaming platforms etc.`
      } catch {
        return 'Your license URL appears to be malformed. Please provide a valid URL to your license text.'
      }
    },
    status: 'required',
    shouldShow: (context: NagContext) => {
      const licenseUrl = context.project.license.url
      if (!licenseUrl) return false

      const isBlocklisted = isUncommonLicenseUrl(licenseUrl, commonLinkDomains.licenseBlocklist)

      try {
        new URL(licenseUrl)
        return isBlocklisted
      } catch {
        return true
      }
    },
    link: {
      path: 'settings',
      title: 'Edit license',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
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
