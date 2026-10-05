import type { Labrinth } from '@modrinth/api-client'
import { DatabaseIcon } from '@modrinth/assets'
import { ENVIRONMENTS_COPY } from '@modrinth/ui'

import { requiresEnvironmentInfo } from '../../utils'
import environmentCorrectionMessage from '../messages/checklist/messages/metadata/environment/correction.md'
import environmentInaccurateMessage from '../messages/checklist/messages/metadata/environment/inaccurate.md'
import environmentMixedMessage from '../messages/checklist/messages/metadata/environment/mixed.md'
import gameVersionsMessage from '../messages/checklist/messages/metadata/game-versions.md'
import loadersMessage from '../messages/checklist/messages/metadata/loaders.md'
import { issue, panel, section, select, toggle } from './component-builders/builders'
import { issueLocation } from './component-builders/locations'

const environments = Object.keys(ENVIRONMENTS_COPY).filter(
	(id) => id !== 'unknown',
) as Labrinth.Projects.v3.Environment[]

export const metadataEnvironmentIssue = issue({
	id: 'metadata-environment',
	locations: [issueLocation('versions')],
	title: 'Incorrect environment',
	category: 'Metadata',
	message: ({ getSelectValue }) => {
		const environment = getSelectValue('correct-environment')
		const correction =
			environment === 'mixed'
				? environmentMixedMessage
				: environment !== 'unknown' && Object.hasOwn(ENVIRONMENTS_COPY, environment)
					? environmentCorrectionMessage
							.replaceAll(
								'%SUGGESTED_ENVIRONMENT%',
								() =>
									ENVIRONMENTS_COPY[environment as Labrinth.Projects.v3.Environment].title
										.defaultMessage ?? environment,
							)
							.replaceAll('%SUGGESTED_ENVIRONMENT_ID%', () => encodeURIComponent(environment))
					: ''
		return environmentInaccurateMessage.replaceAll('%CORRECT%', () => correction)
	},
	suggestedStatus: 'flagged',
})

export const metadataGameVersionsIssue = issue({
	id: 'metadata-game-versions',
	locations: [issueLocation('versions')],
	title: 'Incorrect game versions',
	category: 'Metadata',
	message: gameVersionsMessage,
	suggestedStatus: 'flagged',
})

export const metadataLoadersIssue = issue({
	id: 'metadata-loaders',
	locations: [issueLocation('versions')],
	title: 'Incorrect loaders',
	category: 'Metadata',
	message: loadersMessage,
	suggestedStatus: 'rejected',
})

export const metadataReviewPanel = panel({
	title: 'Metadata',
	hint: "Are there any issues with this project's metadata?",
	icon: DatabaseIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e25ee711bf0802d9a9bdb82dce040eb',
	shown: ({ projectV3 }) => !projectV3.minecraft_server,
}).content(
	toggle({
		issue: metadataGameVersionsIssue,
		label: 'Game Versions',
	}),
	toggle({
		issue: metadataLoadersIssue,
		label: 'Loaders',
	}),
	section({
		shown: ({ projectV3 }) => requiresEnvironmentInfo(projectV3.project_types),
	}).content(
		toggle({
			issue: metadataEnvironmentIssue,
			label: 'Environment',
		}),
		section({
			shown: (ctx) => ctx.selected.issueIds.includes(metadataEnvironmentIssue.id),
		}).content(
			select({
				issue: metadataEnvironmentIssue,
				id: 'correct-environment',
				label: 'Correct Environment',
				placeholder: 'Unknown',
				options: [
					...environments.map((value) => ({
						value,
						label: ENVIRONMENTS_COPY[value].title.defaultMessage ?? value,
					})),
					{ value: 'mixed', label: 'Mixed' },
				],
			}),
		),
	),
)
