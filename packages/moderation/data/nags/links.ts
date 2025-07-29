import type { Nag, NagContext } from '../../types/nags'
import { formatProjectType, type Project } from '@modrinth/utils'
import { useVIntl, defineMessage } from '@vintl/vintl'

export const commonLinkDomains = {
  source: ['github.com', 'gitlab.com', 'bitbucket.org', 'codeberg.org', 'git.sr.ht'],
  issues: ['github.com', 'gitlab.com', 'bitbucket.org', 'codeberg.org', 'docs.google.com'],
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

export function isCommonUrl(url: string | null, commonDomains: string[]): boolean {
  if (url === null || url === '') return true
  try {
    const domain = new URL(url).hostname.toLowerCase()
    return commonDomains.some((allowed) => domain.includes(allowed))
  } catch {
    return false
  }
}

export function isUncommonLicenseUrl(url: string | null, domains: string[]): boolean {
  if (url === null || url === '') return false
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
    title: defineMessage({
      id: 'nags.verify-external-links.title',
      defaultMessage: 'Verify external links',
    }),
    description: defineMessage({
      id: 'nags.verify-external-links.description',
      defaultMessage:
        "Some of your external links may be using domains that aren't recognized as common for their link type.",
    }),
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
      title: defineMessage({
        id: 'nags.visit-links-settings.title',
        defaultMessage: 'Visit links settings',
      }),
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-links',
    },
  },
  {
    id: 'invalid-license-url',
    title: defineMessage({
      id: 'nags.invalid-license-url.title',
      defaultMessage: 'Invalid license URL',
    }),
    description: (context: NagContext) => {
      const { formatMessage } = useVIntl()
      const licenseUrl = context.project.license.url

      if (!licenseUrl) {
        return formatMessage(
          defineMessage({
            id: 'nags.invalid-license-url.description.default',
            defaultMessage: 'License URL is invalid.',
          }),
        )
      }

      try {
        const domain = new URL(licenseUrl).hostname.toLowerCase()
        return formatMessage(
          defineMessage({
            id: 'nags.invalid-license-url.description.domain',
            defaultMessage:
              'Your license URL points to {domain}, which is not appropriate for license information. License URLs should link directly to your license file, not social media, gaming platforms etc.',
          }),
          { domain },
        )
      } catch {
        return formatMessage(
          defineMessage({
            id: 'nags.invalid-license-url.description.malformed',
            defaultMessage:
              'Your license URL appears to be malformed. Please provide a valid URL to your license text.',
          }),
        )
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
      title: defineMessage({
        id: 'nags.edit-license.title',
        defaultMessage: 'Edit license',
      }),
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'gpl-license-source-required',
    title: defineMessage({
      id: 'nags.gpl-license-source-required.title',
      defaultMessage: 'License requires source',
    }),
    description: (context: NagContext) => {
      const { formatMessage } = useVIntl()

      return formatMessage(
        defineMessage({
          id: 'nags.gpl-license-source-required.description',
          defaultMessage:
            'Your {projectType} uses a license which requires source code to be available. Please provide a source code link or sources file, or consider using a different license.',
        }),
        {
          projectType: formatProjectType(context.project.project_type).toLowerCase(),
        },
      )
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
        'MPL-2.0',
      ]

      const isGplLicense = gplLicenses.includes(context.project.license.id)
      const hasSourceUrl = !!context.project.source_url
      const notSourceAsDistributed = (context) => {
        let project = context.project as Project & { actualProjectType: string }
        return context.project.project_type === 'mod' || project.actualProjectType === 'plugin'
      }

      return isGplLicense && notSourceAsDistributed(context) && !hasSourceUrl
    },
    link: {
      path: 'settings/links',
      title: defineMessage({
        id: 'nags.visit-links-settings.title',
        defaultMessage: 'Visit links settings',
      }),
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-links',
    },
  },
]
