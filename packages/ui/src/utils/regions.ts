import { defineMessage, type MessageDescriptor } from '@vintl/vintl'

export const regionOverrides = {
  'us-vin': {
    name: defineMessage({ id: 'servers.region.usa', defaultMessage: 'United States of America' }),
    flag: 'https://flagcdn.com/us.svg',
  },
  'eu-lim': {
    name: defineMessage({ id: 'servers.region.european-union', defaultMessage: 'European Union' }),
    flag: 'https://flagcdn.com/eu.svg',
  },
  'kyros-test': {
    name: defineMessage({ id: 'servers.region.test', defaultMessage: 'Antarctica' }),
    flag: 'https://flagcdn.com/aq.svg',
  },
  'de-fra': {
    name: defineMessage({ id: 'servers.region.europe', defaultMessage: 'Europe' }),
    flag: 'https://flagcdn.com/eu.svg',
  },
} satisfies Record<string, { name?: MessageDescriptor; flag?: string }>
