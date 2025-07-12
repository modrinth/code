import type { Nag, NagContext } from '../../types/nags'

export const MIN_DESCRIPTION_CHARS = 500
export const MIN_SUMMARY_CHARS = 125

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
