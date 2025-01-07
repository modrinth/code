import type { RouteLocationRaw } from 'vue-router'

export type LinkTarget = '_blank' | '_self' | '_parent' | '_top' | string

export type LinkType = 'router' | 'external'

export type Linkish = string | RouteLocationRaw | Link

export type Link = {
  destination: string
  type: 'external'
  target: LinkTarget
  callback?: () => void
} | {
  destination: string | RouteLocationRaw
  type: 'router'
  target: LinkTarget
  callback?: () => void
}

export const isLink = (link: unknown): link is Link => {
  return typeof link === 'object' && link !== null && 'destination' in link;
};


export const asLink = (link: Linkish, callback?: () => void): Link => {
  if (isLink(link)) {
    return callback ? { ...link, callback } : link
  }

  if (typeof link !== 'string' || link.startsWith('/')) {
    return { destination: link, type: 'router', target: '_self', callback }
  }

  return { destination: link, type: 'external', target: '_blank', callback }
}
