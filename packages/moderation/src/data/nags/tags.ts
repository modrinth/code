import { defineMessages } from '@modrinth/ui'

import type { NagDefinitions } from './types.ts'

const messages = defineMessages({
	allTitle: { id: 'nags.all-tags-selected.title', defaultMessage: 'Select accurate tags' },
	tooManyTitle: { id: 'nags.too-many-tags.title', defaultMessage: 'Select accurate tags' },
	tooManyServerTitle: {
		id: 'nags.too-many-tags-server.title',
		defaultMessage: 'Select accurate tags',
	},
	all: {
		id: 'nags.all-tags-selected.description',
		defaultMessage: `You've selected all {totalAvailableTags, plural, one {# available tag} other {# available tags}}. This defeats the purpose of tags, which are meant to help users find relevant projects. Please select only the tags that are relevant to your project.`,
	},
	resolutionTitle: {
		id: 'nags.multiple-resolution-tags.title',
		defaultMessage: 'Select correct resolution',
	},
	resolution: {
		id: 'nags.multiple-resolution-tags.description',
		defaultMessage: `You've selected {count, plural, one {# resolution tag} other {# resolution tags}} ({tags}). Resource packs should typically only have one resolution tag that matches their primary resolution.`,
	},
	selectTitle: { id: 'nags.select-tags.title', defaultMessage: 'Select tags' },
	select: {
		id: 'nags.select-tags.description',
		defaultMessage:
			'Select the tags that correctly apply to your project to help the right users find it.',
	},
	tooMany: {
		id: 'nags.too-many-tags.description',
		defaultMessage: `You've selected {tagCount, plural, one {# tag} other {# tags}}. Consider reducing to {maxTagCount} or fewer to make sure your project appears in relevant search results.`,
	},
	tooManyServer: {
		id: 'nags.too-many-tags-server.description',
		defaultMessage: `You've selected {tagCount, plural, one {# tag} other {# tags}}. Please reduce to {maxTagCount} or fewer to make sure your server appears in relevant search results.`,
	},
	editTags: { id: 'nags.edit-tags.title', defaultMessage: 'Edit tags' },
})

export const tagNags = {
	'all-tags-selected': {
		title: messages.allTitle,
		description: messages.all,
		destination: 'tags',
		linkTitle: messages.editTags,
	},
	'multiple-resolution-tags': {
		title: messages.resolutionTitle,
		description: messages.resolution,
		destination: 'tags',
		linkTitle: messages.editTags,
	},
	'select-tags': { title: messages.selectTitle, description: messages.select, destination: 'tags' },
	'too-many-tags': {
		title: messages.tooManyTitle,
		description: messages.tooMany,
		destination: 'tags',
		linkTitle: messages.editTags,
	},
	'too-many-tags-server': {
		title: messages.tooManyServerTitle,
		description: messages.tooManyServer,
		destination: 'tags',
		linkTitle: messages.editTags,
	},
} satisfies NagDefinitions
