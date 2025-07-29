import { renderHighlightedString } from '@modrinth/utils'
import type { Nag, NagContext } from '../../types/nags'
import { useVIntl, defineMessage } from '@vintl/vintl'

export const MIN_DESCRIPTION_CHARS = 500
export const MAX_HEADER_LENGTH = 100
export const MIN_SUMMARY_CHARS = 35

export function analyzeHeaderLength(markdown: string): {
  hasLongHeaders: boolean
  longHeaders: string[]
} {
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

export function analyzeImageContent(markdown: string): {
  imageHeavy: boolean
  hasEmptyAltText: boolean
} {
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

export function countText(markdown: string): number {
  const htmlString = renderHighlightedString(markdown)
  const parser = new DOMParser()
  const doc = parser.parseFromString(htmlString, 'text/html')
  const walker = document.createTreeWalker(doc, NodeFilter.SHOW_TEXT)

  const textList: string[] = []
  let currentNode: Node | null = walker.currentNode

  while (currentNode) {
    if (currentNode.textContent !== null) {
      textList.push(currentNode.textContent)
    }
    currentNode = walker.nextNode()
  }

  return textList.join(' ').trim().length
}

export const descriptionNags: Nag[] = [
  {
    id: 'description-too-short',
    title: defineMessage({
      id: 'nags.description-too-short.title',
      defaultMessage: 'Description may be insufficient',
    }),
    description: (context: NagContext) => {
      const { formatMessage } = useVIntl()
      const readableLength = countText(context.project.body || '')

      return formatMessage(
        defineMessage({
          id: 'nags.description-too-short.description',
          defaultMessage:
            "Your description is {length} readable characters. It's recommended to have at least {minChars} readable characters to provide users with enough information about your project.",
        }),
        {
          length: readableLength,
          minChars: MIN_DESCRIPTION_CHARS,
        },
      )
    },
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const readableLength = countText(context.project.body || '')
      return readableLength < MIN_DESCRIPTION_CHARS && readableLength > 0
    },
    link: {
      path: 'settings/description',
      title: defineMessage({
        id: 'nags.edit-description.title',
        defaultMessage: 'Edit description',
      }),
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
    },
  },
  {
    id: 'long-headers',
    title: defineMessage({
      id: 'nags.long-headers.title',
      defaultMessage: 'Headers are too long',
    }),
    description: (context: NagContext) => {
      const { formatMessage } = useVIntl()
      const { longHeaders } = analyzeHeaderLength(context.project.body || '')
      const count = longHeaders.length

      return formatMessage(
        defineMessage({
          id: 'nags.long-headers.description',
          defaultMessage:
            '{count, plural, one {# header} other {# headers}} in your description {count, plural, one {is} other {are}} too long. Headers should be concise and act as section titles, not full sentences.',
        }),
        {
          count,
        },
      )
    },
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const { hasLongHeaders } = analyzeHeaderLength(context.project.body || '')
      return hasLongHeaders
    },
    link: {
      path: 'settings/description',
      title: defineMessage({
        id: 'nags.edit-description.title',
        defaultMessage: 'Edit description',
      }),
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
    },
  },
  {
    id: 'summary-too-short',
    title: defineMessage({
      id: 'nags.summary-too-short.title',
      defaultMessage: 'Summary may be insufficient',
    }),
    description: (context: NagContext) => {
      const { formatMessage } = useVIntl()

      return formatMessage(
        defineMessage({
          id: 'nags.summary-too-short.description',
          defaultMessage:
            'Your summary is {length} characters. It should ideally be around {minChars} characters, one short sentence about your project.',
        }),
        {
          length: context.project.description?.length || 0,
          minChars: MIN_SUMMARY_CHARS,
        },
      )
    },
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const summaryLength = context.project.description?.trim()?.length || 0
      return summaryLength < MIN_SUMMARY_CHARS && summaryLength !== 0
    },
    link: {
      path: 'settings',
      title: defineMessage({
        id: 'nags.edit-summary.title',
        defaultMessage: 'Edit summary',
      }),
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'minecraft-title-clause',
    title: defineMessage({
      id: 'nags.minecraft-title-clause.title',
      defaultMessage: 'Title contains "Minecraft"',
    }),
    description: defineMessage({
      id: 'nags.minecraft-title-clause.description',
      defaultMessage:
        'Please remove "Minecraft" from your title. You cannot use "Minecraft" in your title for legal reasons.',
    }),
    status: 'required',
    shouldShow: (context: NagContext) => {
      const title = context.project.title?.toLowerCase() || ''
      const wordsInTitle = title.split(' ').filter((word) => word.length > 0)
      return title.includes('minecraft') && title.length > 0 && wordsInTitle.length <= 3
    },
    link: {
      path: 'settings',
      title: defineMessage({
        id: 'nags.edit-title.title',
        defaultMessage: 'Edit title',
      }),
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'title-contains-technical-info',
    title: defineMessage({
      id: 'nags.title-contains-technical-info.title',
      defaultMessage: 'Title contains loader or version info',
    }),
    description: defineMessage({
      id: 'nags.title-contains-technical-info.description',
      defaultMessage:
        'Removing these helps keep titles clean and makes your project easier to find. Version and loader information is automatically displayed alongside your project.',
    }),
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
      title: defineMessage({
        id: 'nags.edit-title.title',
        defaultMessage: 'Edit title',
      }),
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'summary-same-as-title',
    title: defineMessage({
      id: 'nags.summary-same-as-title.title',
      defaultMessage: 'Summary is project name',
    }),
    description: defineMessage({
      id: 'nags.summary-same-as-title.description',
      defaultMessage:
        "Your summary is the same as your project name. Please change it. It's recommended to have a unique summary to provide more context about your project.",
    }),
    status: 'required',
    shouldShow: (context: NagContext) => {
      const title = context.project.title?.trim() || ''
      const summary = context.project.description?.trim() || ''
      return title === summary && title.length > 0 && summary.length > 0
    },
    link: {
      path: 'settings',
      title: defineMessage({
        id: 'nags.edit-summary.title',
        defaultMessage: 'Edit summary',
      }),
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'image-heavy-description',
    title: defineMessage({
      id: 'nags.image-heavy-description.title',
      defaultMessage: 'Description is mostly images',
    }),
    description: defineMessage({
      id: 'nags.image-heavy-description.description',
      defaultMessage:
        'Please add more descriptive text to help users understand your project, especially those using screen readers or with slow internet connections.',
    }),
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const { imageHeavy } = analyzeImageContent(context.project.body || '')
      return imageHeavy
    },
    link: {
      path: 'settings/description',
      title: defineMessage({
        id: 'nags.edit-description.title',
        defaultMessage: 'Edit description',
      }),
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
    },
  },
  {
    id: 'missing-alt-text',
    title: defineMessage({
      id: 'nags.missing-alt-text.title',
      defaultMessage: 'Images missing alt text',
    }),
    description: defineMessage({
      id: 'nags.missing-alt-text.description',
      defaultMessage:
        'Some of your images are missing alt text, which is important for accessibility, especially for visually impaired users.',
    }),
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const { hasEmptyAltText } = analyzeImageContent(context.project.body || '')
      return hasEmptyAltText
    },
    link: {
      path: 'settings/description',
      title: defineMessage({
        id: 'nags.edit-description.title',
        defaultMessage: 'Edit description',
      }),
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
    },
  },
]
