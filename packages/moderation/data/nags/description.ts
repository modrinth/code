import type { Nag, NagContext } from '../../types/nags'

export const MIN_DESCRIPTION_CHARS = 500
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
    const isVeryLong = headerText.length > 100
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
    title: 'Description may be insufficient',
    description: (context: NagContext) =>
      `Your description is ${context.project.body?.length || 0} characters. It's recommended to have at least ${MIN_DESCRIPTION_CHARS} characters to provide users with enough information about your project.`,
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const bodyLength = context.project.body?.trim()?.length || 0
      return bodyLength < MIN_DESCRIPTION_CHARS && bodyLength !== 0
    },
    link: {
      path: 'settings/description',
      title: 'Edit description',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
    },
  },
  {
    id: 'long-headers',
    title: 'Headers are too long',
    description: (context: NagContext) => {
      const { longHeaders } = analyzeHeaderLength(context.project.body || '')
      const count = longHeaders.length

      return `${count} header${count > 1 ? 's' : ''} in your description ${count > 1 ? 'are' : 'is'} too long. Headers should be concise and act as section titles, not full sentences.`
    },
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const { hasLongHeaders } = analyzeHeaderLength(context.project.body || '')
      return hasLongHeaders
    },
    link: {
      path: 'settings/description',
      title: 'Edit description',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
    },
  },
  {
    id: 'summary-too-short',
    title: 'Summary may be insufficient',
    description: (context: NagContext) =>
      `Your summary is ${context.project.description?.length || 0} characters. It's recommended to have at least ${MIN_SUMMARY_CHARS} characters to provide users with enough information about your project.`,
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const summaryLength = context.project.description?.trim()?.length || 0
      return summaryLength < MIN_SUMMARY_CHARS && summaryLength !== 0
    },
    link: {
      path: 'settings',
      title: 'Edit summary',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'minecraft-title-clause',
    title: 'Title contains "Minecraft"',
    description: () =>
      `Please remove "Minecraft" from your title. You cannot use "Minecraft" in your title for legal reasons.`,
    status: 'required',
    shouldShow: (context: NagContext) => {
      const title = context.project.title?.toLowerCase() || ''
      return title.includes('minecraft') && title.length > 0
    },
    link: {
      path: 'settings',
      title: 'Edit title',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'title-contains-technical-info',
    title: 'Title contains loader or version info',
    description: () => {
      return `Removing these helps keep titles clean and makes your project easier to find. Version and loader information is automatically displayed alongside your project.`
    },
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
      title: 'Edit title',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'summary-same-as-title',
    title: 'Summary is project name',
    description: () =>
      `Your summary is the same as your project name. Please change it. It's recommended to have a unique summary to provide more context about your project.`,
    status: 'required',
    shouldShow: (context: NagContext) => {
      const title = context.project.title?.trim() || ''
      const summary = context.project.description?.trim() || ''
      return title === summary && title.length > 0 && summary.length > 0
    },
    link: {
      path: 'settings',
      title: 'Edit summary',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
    },
  },
  {
    id: 'image-heavy-description',
    title: 'Description is mostly images',
    description: () =>
      `Please add more descriptive text to help users understand your project, especially those using screen readers or with slow internet connections.`,
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const { imageHeavy } = analyzeImageContent(context.project.body || '')
      return imageHeavy
    },
    link: {
      path: 'settings/description',
      title: 'Edit description',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
    },
  },
  {
    id: 'missing-alt-text',
    title: 'Images missing alt text',
    description: () =>
      `Some of your images are missing alt text, which is important for accessibility, especially for visually impaired users.`,
    status: 'warning',
    shouldShow: (context: NagContext) => {
      const { hasEmptyAltText } = analyzeImageContent(context.project.body || '')
      return hasEmptyAltText
    },
    link: {
      path: 'settings/description',
      title: 'Edit description',
      shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-description',
    },
  },
]
