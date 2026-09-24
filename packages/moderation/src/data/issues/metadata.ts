import type { Labrinth } from '@modrinth/api-client'
import { DatabaseIcon } from '@modrinth/assets'
import { ENVIRONMENTS_COPY } from '@modrinth/ui'

import { requiresEnvironmentInfo } from '../../utils'
import dependenciesMessage from '../messages/checklist/messages/metadata/dependencies.md'
import environmentCorrectionMessage from '../messages/checklist/messages/metadata/environment/correction.md'
import environmentInaccurateMessage from '../messages/checklist/messages/metadata/environment/inaccurate.md'
import environmentMixedMessage from '../messages/checklist/messages/metadata/environment/mixed.md'
import gameVersionsMessage from '../messages/checklist/messages/metadata/game-versions.md'
import loadersMessage from '../messages/checklist/messages/metadata/loaders.md'
import { issue, panel, section, select, toggle } from './component-builders/builders'

const environments = Object.keys(ENVIRONMENTS_COPY).filter(
	(id) => id !== 'unknown',
) as Labrinth.Projects.v3.Environment[]

export const metadataEnvironmentIssue = issue({
	id: 'metadata-environment',
	title: 'Incorrect environment',
	category: 'Metadata',
	message: ({ getSelectValue }) => {
		const environment = getSelectValue('correct-environment')
		const correction =
			environment === 'mixed'
				? environmentMixedMessage
				: environment
					? environmentCorrectionMessage.replaceAll(
							'%SUGGESTED_ENVIRONMENT%',
							() =>
								ENVIRONMENTS_COPY[environment as Labrinth.Projects.v3.Environment]?.title
									.defaultMessage ?? environment,
						)
					: ''
		return environmentInaccurateMessage.replaceAll('%CORRECT%', () => correction)
	},
	suggestedStatus: 'flagged',
	corrections: ({ getSelectValue }) => {
		const environment = getSelectValue('correct-environment') as Labrinth.Projects.v3.Environment
		return environments.includes(environment) ? { project: { environment } } : {}
	},
})

export const metadataDependenciesIssue = issue({
	id: 'metadata-dependencies',
	title: 'Incorrect dependencies',
	category: 'Metadata',
	message: dependenciesMessage,
	suggestedStatus: 'flagged',
})

export const metadataGameVersionsIssue = issue({
	id: 'metadata-game-versions',
	title: 'Incorrect game versions',
	category: 'Metadata',
	message: gameVersionsMessage,
	suggestedStatus: 'flagged',
})

export const metadataLoadersIssue = issue({
	id: 'metadata-loaders',
	title: 'Incorrect loaders',
	category: 'Metadata',
	message: loadersMessage,
	suggestedStatus: 'rejected',
})

export const metadataReviewPanel = panel({
	title: 'Metadata',
	hint: "Are there any issues with this project's metadata?",
	icon: DatabaseIcon,
	shown: ({ ProjectV3 }) => !ProjectV3.minecraft_server,
}).content(
	toggle({
		issue: metadataDependenciesIssue,
		label: 'Dependencies',
	}),
	toggle({
		issue: metadataGameVersionsIssue,
		label: 'Game Versions',
	}),
	toggle({
		issue: metadataLoadersIssue,
		label: 'Loaders',
	}),
	section({
		shown: ({ ProjectV3 }) => requiresEnvironmentInfo(ProjectV3.project_types),
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
