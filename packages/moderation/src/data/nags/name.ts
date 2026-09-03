import { defineMessages } from '@modrinth/ui/i18n'

import type { NagDefinitions } from './types.ts'

const messages = defineMessages({
	minecraftTitleClauseTitle: {
		id: 'nags.minecraft-title-clause.title',
		defaultMessage: 'Avoid brand infringement',
	},
	minecraftTitleClauseDescription: {
		id: 'nags.minecraft-title-clause.description',
		defaultMessage: `Projects must not use Minecraft's branding or include "Minecraft" as a significant part of the name.`,
	},
	fixNameTitle: { id: 'nags.invalid-project-name.title', defaultMessage: 'Modify project name' },
	fixVersionTitle: {
		id: 'nags.project-name-version.title',
		defaultMessage: 'Modify project name',
	},
	editName: { id: 'nags.edit-title.title', defaultMessage: 'Edit name' },
	nonStandardText: {
		id: 'nags.project-name-non-standard-text.description',
		defaultMessage:
			'Non-standard text characters, such as “Fancy text” or “Zalgo”, are not allowed in the project name.',
	},
	profanity: {
		id: 'nags.project-name-profanity.description',
		defaultMessage: `Your project's name cannot contain profanity. Detected: “{value}”.`,
	},
	slur: {
		id: 'nags.project-name-slur.description',
		defaultMessage: `Your project's name must not contain offensive terms. Detected: “{value}”.`,
	},
	version: {
		id: 'project.text-validation.title-version-number',
		defaultMessage: 'Project names should not include version numbers.',
	},
})

export const nameNags = {
	'minecraft-title-clause': {
		title: messages.minecraftTitleClauseTitle,
		description: messages.minecraftTitleClauseDescription,
		destination: 'general',
		linkTitle: messages.editName,
	},
	'project-name-non-standard-text': {
		title: messages.fixNameTitle,
		description: messages.nonStandardText,
		destination: 'general',
		linkTitle: messages.editName,
	},
	'project-name-profanity': {
		title: messages.fixNameTitle,
		description: messages.profanity,
		destination: 'general',
		linkTitle: messages.editName,
	},
	'project-name-slur': {
		title: messages.fixNameTitle,
		description: messages.slur,
		destination: 'general',
		linkTitle: messages.editName,
	},
	'project-name-version': {
		title: messages.fixVersionTitle,
		description: messages.version,
		destination: 'general',
		linkTitle: messages.editName,
	},
} satisfies NagDefinitions
