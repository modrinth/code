import { defineMessages } from '@vintl/vintl'

export default defineMessages({
  tooManyTagsTitle: {
    id: 'nags.too-many-tags.title',
    defaultMessage: 'Too many tags selected',
  },
  tooManyTagsDescription: {
    id: 'nags.too-many-tags.description',
    defaultMessage:
      "You've selected {tagCount} tags. Consider reducing to 5 or fewer to keep your project focused and easier to discover.",
  },
  multipleResolutionTagsTitle: {
    id: 'nags.multiple-resolution-tags.title',
    defaultMessage: 'Multiple resolution tags selected',
  },
  multipleResolutionTagsDescription: {
    id: 'nags.multiple-resolution-tags.description',
    defaultMessage:
      "You've selected {count} resolution tags ({tags}). Resource packs should typically only have one resolution tag that matches their primary resolution.",
  },
  allTagsSelectedTitle: {
    id: 'nags.all-tags-selected.title',
    defaultMessage: 'All tags selected',
  },
  allTagsSelectedDescription: {
    id: 'nags.all-tags-selected.description',
    defaultMessage:
      "You've selected all {totalAvailableTags} available tags. This defeats the purpose of tags, which are meant to help users find relevant projects. Please select only the tags that truly apply to your project.",
  },
  editTagsTitle: {
    id: 'nags.edit-tags.title',
    defaultMessage: 'Edit tags',
  },
})
