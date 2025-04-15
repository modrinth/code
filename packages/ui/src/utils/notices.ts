import { defineMessage, type MessageDescriptor } from '@vintl/vintl'

export const NOTICE_LEVELS: Record<
  string,
  { name: MessageDescriptor; colors: { text: string; bg: string } }
> = {
  info: {
    name: defineMessage({
      id: 'servers.notice.level.info.name',
      defaultMessage: 'Info',
    }),
    colors: {
      text: 'var(--color-blue)',
      bg: 'var(--color-blue-bg)',
    },
  },
  warn: {
    name: defineMessage({
      id: 'servers.notice.level.warn.name',
      defaultMessage: 'Warning',
    }),
    colors: {
      text: 'var(--color-orange)',
      bg: 'var(--color-orange-bg)',
    },
  },
  critical: {
    name: defineMessage({
      id: 'servers.notice.level.critical.name',
      defaultMessage: 'Critical',
    }),
    colors: {
      text: 'var(--color-red)',
      bg: 'var(--color-red-bg)',
    },
  },
  survey: {
    name: defineMessage({
      id: 'servers.notice.level.survey.name',
      defaultMessage: 'Survey',
    }),
    colors: {
      text: 'var(--color-purple)',
      bg: 'var(--color-purple-bg)',
    },
  },
}

const DISMISSABLE = {
  name: defineMessage({
    id: 'servers.notice.dismissable',
    defaultMessage: 'Dismissable',
  }),
  colors: {
    text: 'var(--color-green)',
    bg: 'var(--color-green-bg)',
  },
}

const UNDISMISSABLE = {
  name: defineMessage({
    id: 'servers.notice.undismissable',
    defaultMessage: 'Undismissable',
  }),
  colors: {
    text: 'var(--color-red)',
    bg: 'var(--color-red-bg)',
  },
}

export function getDismissableMetadata(dismissable: boolean) {
  return dismissable ? DISMISSABLE : UNDISMISSABLE
}
