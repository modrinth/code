import { VersionIcon } from '@modrinth/assets'

import alternateVersionsAdditionalMessage from '../messages/checklist/messages/versions/alternate-versions/additional.md'
import alternateVersionsMonoMessage from '../messages/checklist/messages/versions/alternate-versions/mono.md'
import alternateVersionsPrimaryMessage from '../messages/checklist/messages/versions/alternate-versions/primary.md'
import alternateVersionsServerMessage from '../messages/checklist/messages/versions/alternate-versions/server.md'
import alternateVersionsServerAdditionalMessage from '../messages/checklist/messages/versions/alternate-versions/server-additional.md'
import alternateVersionsZipMessage from '../messages/checklist/messages/versions/alternate-versions/zip.md'
import duplicatePrimaryFilesMessage from '../messages/checklist/messages/versions/duplicate-primary-files.md'
import incorrectAdditionalFilesMessage from '../messages/checklist/messages/versions/incorrect-additional-files.md'
import incorrectProjectTypeDatapackMessage from '../messages/checklist/messages/versions/incorrect-project-type/datapack.md'
import incorrectProjectTypeModpackMessage from '../messages/checklist/messages/versions/incorrect-project-type/modpack.md'
import incorrectProjectTypeResourcepackMessage from '../messages/checklist/messages/versions/incorrect-project-type/resourcepack.md'
import redistLibsMessage from '../messages/checklist/messages/versions/redist-libs.md'
import unsupportedMessage from '../messages/checklist/messages/versions/unsupported.md'
import vanillaAssetsMessage from '../messages/checklist/messages/versions/vanilla-assets.md'
import { issue, panel, section, select, text, toggle } from './component-builders/builders'

export const versionsIncorrectAdditionalFilesIssue = issue({
	id: 'versions-incorrect-additional-files',
	title: 'Incorrect additional files',
	category: 'Versions',
	message: incorrectAdditionalFilesMessage,
	suggestedStatus: 'flagged',
})

export const versionsVanillaAssetsIssue = issue({
	id: 'versions-vanilla-assets',
	title: 'Vanilla assets',
	category: 'Versions',
	message: vanillaAssetsMessage,
	suggestedStatus: 'rejected',
})

export const versionsRedistLibsIssue = issue({
	id: 'versions-redist-libs',
	title: 'Redistributed libraries',
	category: 'Versions',
	message: redistLibsMessage,
	suggestedStatus: 'rejected',
})

export const versionsDuplicatePrimaryFilesIssue = issue({
	id: 'versions-duplicate-primary-files',
	title: 'Duplicate primary files',
	category: 'Versions',
	message: duplicatePrimaryFilesMessage,
	suggestedStatus: 'flagged',
})

const incorrectProjectTypeMessages: Record<string, string> = {
	modpack: incorrectProjectTypeModpackMessage,
	resourcepack: incorrectProjectTypeResourcepackMessage,
	datapack: incorrectProjectTypeDatapackMessage,
}

export const versionsIncorrectProjectTypeIssue = issue({
	id: 'versions-incorrect-project-type',
	title: 'Incorrect project type',
	category: 'Versions',
	message: ({ getSelectValue }) => incorrectProjectTypeMessages[getSelectValue('type')] ?? '',
	suggestedStatus: 'rejected',
})

const alternateVersionsMessages: Record<string, string> = {
	primary: alternateVersionsPrimaryMessage,
	additional: alternateVersionsAdditionalMessage,
	mono: alternateVersionsMonoMessage,
	server: alternateVersionsServerMessage,
	'server-additional': alternateVersionsServerAdditionalMessage,
	zip: alternateVersionsZipMessage,
}

export const versionsAlternateVersionsIssue = issue({
	id: 'versions-alternate-versions',
	title: 'Alternate versions',
	category: 'Versions',
	message: ({ getSelectValue }) => alternateVersionsMessages[getSelectValue('distribution')] ?? '',
	suggestedStatus: 'rejected',
})

export const versionsUnsupportedIssue = issue({
	id: 'versions-unsupported',
	title: 'Unsupported versions',
	category: 'Versions',
	message: ({ getTextValue }) =>
		unsupportedMessage.replaceAll('%INVALID_TYPE%', () => getTextValue('invalid-type')),
	suggestedStatus: 'rejected',
})

export const versionsReviewPanel = panel({
	hint: "Are this project's files correct?",
	icon: VersionIcon,
	shown: ({ ProjectV3 }) => !ProjectV3.minecraft_server,
}).content(
	toggle({
		issue: versionsIncorrectAdditionalFilesIssue,
		label: 'Incorrect additional files',
	}),
	toggle({
		issue: versionsVanillaAssetsIssue,
		label: 'Vanilla Assets',
		shown: ({ ProjectV3 }) => ProjectV3.project_types.includes('resourcepack'),
	}),
	toggle({
		issue: versionsRedistLibsIssue,
		label: 'Packed Libs',
		shown: ({ ProjectV3 }) =>
			ProjectV3.project_types.includes('mod') || ProjectV3.project_types.includes('plugin'),
	}),
	toggle({
		issue: versionsDuplicatePrimaryFilesIssue,
		label: 'Duplicate Primary Files',
	}),
	toggle({
		issue: versionsIncorrectProjectTypeIssue,
		label: 'Incorrect Project Type',
	}),
	section({
		shown: (ctx) => ctx.selected.issueIds.includes(versionsIncorrectProjectTypeIssue.id),
	}).content(
		select({
			issue: versionsIncorrectProjectTypeIssue,
			id: 'type',
			label: 'Correct Project Type',
			required: true,
			placeholder: 'Unknown',
			options: [
				{ value: 'modpack', label: 'Modpack' },
				{ value: 'resourcepack', label: 'Resource Pack' },
				{ value: 'datapack', label: 'Data Pack' },
			],
		}),
	),
	toggle({
		issue: versionsAlternateVersionsIssue,
		label: 'Alternate Versions',
	}),
	section({
		shown: (ctx) => ctx.selected.issueIds.includes(versionsAlternateVersionsIssue.id),
	}).content(
		select({
			issue: versionsAlternateVersionsIssue,
			id: 'distribution',
			label: 'Distribution Type',
			required: true,
			placeholder: 'Unknown',
			options: [
				{ value: 'primary', label: 'Primary Files' },
				{ value: 'additional', label: 'Additional Files' },
				{
					value: 'mono',
					label: 'Monofile',
					shown: ({ ProjectV3 }) =>
						ProjectV3.project_types.includes('resourcepack') ||
						ProjectV3.loaders.includes('datapack'),
				},
				{
					value: 'server',
					label: 'Server Files (Primary Files)',
					shown: ({ ProjectV3 }) => ProjectV3.project_types.includes('modpack'),
				},
				{
					value: 'server-additional',
					label: 'Server Files (Additional Files)',
					shown: ({ ProjectV3 }) => ProjectV3.project_types.includes('modpack'),
				},
				{
					value: 'zip',
					label: 'mods.zip',
					shown: ({ ProjectV3 }) => ProjectV3.project_types.includes('modpack'),
				},
			],
		}),
	),
	toggle({ issue: versionsUnsupportedIssue, label: 'Unsupported' }),
	section({
		shown: (ctx) => ctx.selected.issueIds.includes(versionsUnsupportedIssue.id),
	}).content(
		text({
			issue: versionsUnsupportedIssue,
			id: 'invalid-type',
			label: 'Unsupported Type',
			required: true,
		}),
	),
)
