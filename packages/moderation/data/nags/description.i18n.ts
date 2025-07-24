import { defineMessages } from '@vintl/vintl'

export default defineMessages({
  descriptionTooShortTitle: {
    id: 'nags.description-too-short.title',
    defaultMessage: 'Description may be insufficient',
  },
  descriptionTooShortDescription: {
    id: 'nags.description-too-short.description',
    defaultMessage:
      "Your description is {length} characters. It's recommended to have at least {minChars} characters to provide users with enough information about your project.",
  },
  longHeadersTitle: {
    id: 'nags.long-headers.title',
    defaultMessage: 'Headers are too long',
  },
  longHeadersDescription: {
    id: 'nags.long-headers.description',
    defaultMessage:
      '{count, plural, one {# header} other {# headers}} in your description {count, plural, one {is} other {are}} too long. Headers should be concise and act as section titles, not full sentences.',
  },
  summaryTooShortTitle: {
    id: 'nags.summary-too-short.title',
    defaultMessage: 'Summary may be insufficient',
  },
  summaryTooShortDescription: {
    id: 'nags.summary-too-short.description',
    defaultMessage:
      "Your summary is {length} characters. It's recommended to have at least {minChars} characters to provide users with enough information about your project.",
  },
  minecraftTitleClauseTitle: {
    id: 'nags.minecraft-title-clause.title',
    defaultMessage: 'Title contains "Minecraft"',
  },
  minecraftTitleClauseDescription: {
    id: 'nags.minecraft-title-clause.description',
    defaultMessage:
      'Please remove "Minecraft" from your title. You cannot use "Minecraft" in your title for legal reasons.',
  },
  titleContainsTechnicalInfoTitle: {
    id: 'nags.title-contains-technical-info.title',
    defaultMessage: 'Title contains loader or version info',
  },
  titleContainsTechnicalInfoDescription: {
    id: 'nags.title-contains-technical-info.description',
    defaultMessage:
      'Removing these helps keep titles clean and makes your project easier to find. Version and loader information is automatically displayed alongside your project.',
  },
  summarySameAsTitleTitle: {
    id: 'nags.summary-same-as-title.title',
    defaultMessage: 'Summary is project name',
  },
  summarySameAsTitleDescription: {
    id: 'nags.summary-same-as-title.description',
    defaultMessage:
      "Your summary is the same as your project name. Please change it. It's recommended to have a unique summary to provide more context about your project.",
  },
  imageHeavyDescriptionTitle: {
    id: 'nags.image-heavy-description.title',
    defaultMessage: 'Description is mostly images',
  },
  imageHeavyDescriptionDescription: {
    id: 'nags.image-heavy-description.description',
    defaultMessage:
      'Please add more descriptive text to help users understand your project, especially those using screen readers or with slow internet connections.',
  },
  missingAltTextTitle: {
    id: 'nags.missing-alt-text.title',
    defaultMessage: 'Images missing alt text',
  },
  missingAltTextDescription: {
    id: 'nags.missing-alt-text.description',
    defaultMessage:
      'Some of your images are missing alt text, which is important for accessibility, especially for visually impaired users.',
  },
  editDescriptionTitle: {
    id: 'nags.edit-description.title',
    defaultMessage: 'Edit description',
  },
  editSummaryTitle: {
    id: 'nags.edit-summary.title',
    defaultMessage: 'Edit summary',
  },
  editTitleTitle: {
    id: 'nags.edit-title.title',
    defaultMessage: 'Edit title',
  },
})
