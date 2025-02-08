export interface BuiltinLicense {
  friendly: string
  short: string
  requiresOnlyOrLater?: boolean
}

export const builtinLicenses: BuiltinLicense[] = [
  { friendly: 'Custom', short: '' },
  {
    friendly: 'All Rights Reserved/No License',
    short: 'All-Rights-Reserved',
  },
  { friendly: 'Apache License 2.0', short: 'Apache-2.0' },
  {
    friendly: 'BSD 2-Clause "Simplified" License',
    short: 'BSD-2-Clause',
  },
  {
    friendly: 'BSD 3-Clause "New" or "Revised" License',
    short: 'BSD-3-Clause',
  },
  {
    friendly: 'CC Zero (Public Domain equivalent)',
    short: 'CC0-1.0',
  },
  { friendly: 'CC-BY 4.0', short: 'CC-BY-4.0' },
  {
    friendly: 'CC-BY-SA 4.0',
    short: 'CC-BY-SA-4.0',
  },
  {
    friendly: 'CC-BY-NC 4.0',
    short: 'CC-BY-NC-4.0',
  },
  {
    friendly: 'CC-BY-NC-SA 4.0',
    short: 'CC-BY-NC-SA-4.0',
  },
  {
    friendly: 'CC-BY-ND 4.0',
    short: 'CC-BY-ND-4.0',
  },
  {
    friendly: 'CC-BY-NC-ND 4.0',
    short: 'CC-BY-NC-ND-4.0',
  },
  {
    friendly: 'GNU Affero General Public License v3',
    short: 'AGPL-3.0',
    requiresOnlyOrLater: true,
  },
  {
    friendly: 'GNU Lesser General Public License v2.1',
    short: 'LGPL-2.1',
    requiresOnlyOrLater: true,
  },
  {
    friendly: 'GNU Lesser General Public License v3',
    short: 'LGPL-3.0',
    requiresOnlyOrLater: true,
  },
  {
    friendly: 'GNU General Public License v2',
    short: 'GPL-2.0',
    requiresOnlyOrLater: true,
  },
  {
    friendly: 'GNU General Public License v3',
    short: 'GPL-3.0',
    requiresOnlyOrLater: true,
  },
  { friendly: 'ISC License', short: 'ISC' },
  { friendly: 'MIT License', short: 'MIT' },
  { friendly: 'Mozilla Public License 2.0', short: 'MPL-2.0' },
  { friendly: 'zlib License', short: 'Zlib' },
] as const
