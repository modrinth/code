import type { Nag, NagContext } from '../../types/nags'
import { useVIntl } from '@vintl/vintl'

import messages from './description.i18n'

export const MIN_DESCRIPTION_CHARS = 500
export const MAX_HEADER_LENGTH = 100
export const MIN_SUMMARY_CHARS = 125

function analyzeHeaderLength(markdown: string): { hasLongHeaders: boolean; longHeaders: string[] } {
  if (!markdown) return { hasLongHeaders: false, longHeaders: [] }

  const withoutCodeBlocks = markdown.replace(/```[\s\S]*?```/g, '').replace(/`[^`]*`/g, '')

  const headerRegex = /^(#{1,3})\s+(.+)$/gm
  const headers = [...withoutCodeBlocks.matchAll(headerRegex)]

  const longHeaders: string[] = []

  headers.forEach((match) => {
    const headerText = match[2].trim()
    const sentenceEnders = /[.!?]+/g
    const sentences = headerText.split(sentenceEnders).filter((s) => s.trim().length > 0)

    const hasSentenceEnders = sentenceEnders.test(headerText)
    const isVeryLong = headerText.length > MAX_HEADER_LENGTH
    const hasMultipleSentences = sentences.length > 1

    if (hasSentenceEnders || isVeryLong || hasMultipleSentences) {
      longHeaders.push(headerText)
    }
  })

  return {
    hasLongHeaders: longHeaders.length > 0,
    longHeaders,
  }
}

function analyzeImageContent(markdown: string): { imageHeavy: boolean; hasEmptyAltText: boolean } {
  if (!markdown) return { imageHeavy: false, hasEmptyAltText: false }

  const withoutCodeBlocks = markdown.replace(/```[\s\S]*?```/g, '').replace(/`[^`]*`/g, '')

  const imageRegex = /!\[([^\]]*)\]\([^)]+\)/g
  const images = [...withoutCodeBlocks.matchAll(imageRegex)]

  const htmlImageRegex = /<img[^>]*>/gi
  const htmlImages = [...withoutCodeBlocks.matchAll(htmlImageRegex)]

  const totalImages = images.length + htmlImages.length
  if (totalImages === 0) return { imageHeavy: false, hasEmptyAltText: false }

  const textWithoutImages = withoutCodeBlocks
    .replace(/!\[([^\]]*)\]\([^)]+\)/g, '')
    .replace(/<img[^>]*>/gi, '')
    .replace(/\s+/g, ' ')
    .trim()

  const textLength = textWithoutImages.length
  const imageHeavy = textLength < 100 || (totalImages >= 3 && textLength < 200)

  const hasEmptyAltText =
    images.some((match) => !match[1]?.trim()) ||
    htmlImages.some((match) => {
      const altMatch = match[0].match(/alt\s*=\s*["']([^"']*)["']/i)
      return !altMatch || !altMatch[1]?.trim()
    })

  return { imageHeavy, hasEmptyAltText }
}

export const descriptionNags: Nag[] = [
  {
    id: 'description-too-short',
    title: messages.descriptionTooShortTitle,
    description: (context: NagContext) => {
      const { formatMessage } = useVIntl()

      return formatMessage(messages.descriptionTooShortDescription, {
        length: context.project.body?.length || 0,
        minChars: MIN_DESCRIPTION_CHARS,
      })
    },
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const bodyLength = context.project.body?.trim()?.length || 0
      return bodyLength < MIN_DESCRIPTION_CHARS && bodyLength !== 0
    },
    link: {
      path: 'settings/description',
      title: messages.editDescriptionTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
    },
  },
  {
    id: 'long-headers',
    title: messages.longHeadersTitle,
    description: (context: NagContext) => {
      const { formatMessage } = useVIntl()
      const { longHeaders } = analyzeHeaderLength(context.project.body || '')
      const count = longHeaders.length

      return formatMessage(messages.longHeadersDescription, {
        count,
      })
    },
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const { hasLongHeaders } = analyzeHeaderLength(context.project.body || '')
      return hasLongHeaders
    },
    link: {
      path: 'settings/description',
      title: messages.editDescriptionTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
    },
  },
  {
    id: 'summary-too-short',
    title: messages.summaryTooShortTitle,
    description: (context: NagContext) => {
      const { formatMessage } = useVIntl()

      return formatMessage(messages.summaryTooShortDescription, {
        length: context.project.description?.length || 0,
        minChars: MIN_SUMMARY_CHARS,
      })
    },
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const summaryLength = context.project.description?.trim()?.length || 0
      return summaryLength < MIN_SUMMARY_CHARS && summaryLength !== 0
    },
    link: {
      path: 'settings',
      title: messages.editSummaryTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'minecraft-title-clause',
    title: messages.minecraftTitleClauseTitle,
    description: messages.minecraftTitleClauseDescription,
    status: 'required',
    shouldShow: (context: NagContext) => {
      const title = context.project.title?.toLowerCase() || ''
      return title.includes('minecraft') && title.length > 0
    },
    link: {
      path: 'settings',
      title: messages.editTitleTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'title-contains-technical-info',
    title: messages.titleContainsTechnicalInfoTitle,
    description: messages.titleContainsTechnicalInfoDescription,
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const title = context.project.title?.toLowerCase() || ''
      if (!title) return false

      const loaderNames =
        context.tags.loaders?.map((loader: { name: string }) => loader.name?.toLowerCase()) || []
      const hasLoader = loaderNames.some((loader) => loader && title.includes(loader.toLowerCase()))
      const versionPatterns = [/\b1\.\d+(\.\d+)?\b/]
      const hasVersionPattern = versionPatterns.some((pattern) => pattern.test(title))

      return hasLoader || hasVersionPattern
    },
    link: {
      path: 'settings',
      title: messages.editTitleTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'summary-same-as-title',
    title: messages.summarySameAsTitleTitle,
    description: messages.summarySameAsTitleDescription,
    status: 'required',
    shouldShow: (context: NagContext) => {
      const title = context.project.title?.trim() || ''
      const summary = context.project.description?.trim() || ''
      return title === summary && title.length > 0 && summary.length > 0
    },
    link: {
      path: 'settings',
      title: messages.editSummaryTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'image-heavy-description',
    title: messages.imageHeavyDescriptionTitle,
    description: messages.imageHeavyDescriptionDescription,
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const { imageHeavy } = analyzeImageContent(context.project.body || '')
      return imageHeavy
    },
    link: {
      path: 'settings/description',
      title: messages.editDescriptionTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
    },
  },
  {
    id: 'missing-alt-text',
    title: messages.missingAltTextTitle,
    description: messages.missingAltTextDescription,
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const { hasEmptyAltText } = analyzeImageContent(context.project.body || '')
      return hasEmptyAltText
    },
    link: {
      path: 'settings/description',
      title: messages.editDescriptionTitle,
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
    },
  },
]
