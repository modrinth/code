import type { Nag, NagContext } from '../../types/nags'
import { formatProjectType } from '@modrinth/utils'
import { useVIntl } from '@vintl/vintl'

import messages from './links.i18n'

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
    title: messages.verifyExternalLinksTitle,
    description: messages.verifyExternalLinksDescription,
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
      title: messages.visitLinksSettingsTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-links',
    },
  },
  {
    id: 'invalid-license-url',
    title: messages.invalidLicenseUrlTitle,
    description: (context: NagContext) => {
      const { formatMessage } = useVIntl()
      const licenseUrl = context.project.license.url

      if (!licenseUrl) {
        return formatMessage(messages.invalidLicenseUrlDescriptionDefault)
      }

      try {
        const domain = new URL(licenseUrl).hostname.toLowerCase()
        return formatMessage(messages.invalidLicenseUrlDescriptionDomain, { domain })
      } catch {
        return formatMessage(messages.invalidLicenseUrlDescriptionMalformed)
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
      title: messages.editLicenseTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'gpl-license-source-required',
    title: messages.gplLicenseSourceRequiredTitle,
    description: (context: NagContext) => {
      const { formatMessage } = useVIntl()

      return formatMessage(messages.gplLicenseSourceRequiredDescription, {
        projectType: formatProjectType(context.project.project_type).toLowerCase(),
      })
    },
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
      title: messages.visitLinksSettingsTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-links',
    },
  },
]
