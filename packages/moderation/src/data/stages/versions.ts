import { VersionIcon } from '@modrinth/assets'

import type {
	ButtonAction,
	ChecklistActionContext,
	DropdownAction,
	DropdownActionOption,
	MultiSelectChipsAction,
	MultiSelectChipsOption,
} from '../../types/actions'
import type { Stage } from '../../types/stage'

const loaderLabels: Record<string, string> = {
	datapack: 'Data Pack',
	resourcepack: 'Resource Pack',
	neoforge: 'NeoForge',
	liteloader: 'LiteLoader',
}

function formatLoaderLabel(loader: string): string {
	if (loaderLabels[loader]) {
		return loaderLabels[loader]
	}

	return loader
		.split(/[-_]/g)
		.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
		.join(' ')
}

function getIncorrectLoaderOptions(context: ChecklistActionContext): MultiSelectChipsOption[] {
	const distinctLoaders = [...new Set((context.versions ?? []).flatMap((version) => version.loaders ?? []))]

	return distinctLoaders
		.sort((a, b) => formatLoaderLabel(a).localeCompare(formatLoaderLabel(b)))
		.map((loader, index) => ({
			id: loader,
			label: formatLoaderLabel(loader),
			weight: 1002 + index,
			message: async () => `- ${formatLoaderLabel(loader)}`,
		}))
}

const versions: Stage = {
	title: 'Versions',
	hint: "Are this project's files correct?",
	id: 'versions',
	icon: VersionIcon,
	guidance_url:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e25ee711bf0804bad38e9055951ff31',
	navigate: '/versions',
	shouldShow: (project, projectV3) => !projectV3?.minecraft_server,
	actions: [
		{
			id: 'versions_incorrect_additional',
			type: 'button',
			label: 'Incorrect additional files',
			weight: 1000,
			suggestedStatus: 'flagged',
			severity: 'medium',
			message: async () =>
				(await import('../messages/checklist-messages/versions/incorrect_additional_files.md?raw'))
					.default,
		} as ButtonAction,
		{
			id: 'versions_incorrect_project_type',
			type: 'button',
			label: 'Incorrect Project Type',
			suggestedStatus: 'rejected',
			severity: 'medium',
			weight: -999999,
			message: async () => '',
			enablesActions: [
				{
					id: 'versions_incorrect_project_type_options',
					type: 'dropdown',
					label: 'What type should this project be?',
					options: [
						{
							label: 'Modpack',
							weight: 1001,
							shouldShow: (project) => project.project_type !== 'modpack',
							message: async () =>
								(await import('../messages/checklist-messages/versions/invalid-modpacks.md?raw'))
									.default,
						} as DropdownActionOption,
						{
							label: 'Resource Pack',
							weight: 1001,
							shouldShow: (project) => project.project_type !== 'resourcepack',
							message: async () =>
								(
									await import('../messages/checklist-messages/versions/invalid-resourcepacks.md?raw')
								).default,
						} as DropdownActionOption,
						{
							label: 'Data Pack',
							weight: 1001,
							shouldShow: (project) => !project.loaders.includes('datapack'),
							message: async () =>
								(await import('../messages/checklist-messages/versions/invalid-datapacks.md?raw'))
									.default,
						} as DropdownActionOption,
					],
				} as DropdownAction,
			],
		} as ButtonAction,
		{
			id: 'versions_alternate_versions',
			type: 'button',
			label: 'Alternate Versions',
			suggestedStatus: 'flagged',
			severity: 'medium',
			weight: -999999,
			message: async () => '',
			enablesActions: [
				{
					id: 'versions_alternate_versions_options',
					type: 'dropdown',
					label: 'How are the alternate versions distributed?',
					options: [
						{
							label: 'Primary Files',
							weight: 1002,
							message: async () =>
								(
									await import('../messages/checklist-messages/versions/alternate_versions-primary.md?raw')
								).default,
						} as DropdownActionOption,
						{
							label: 'Additional Files',
							weight: 1002,
							message: async () =>
								(
									await import('../messages/checklist-messages/versions/alternate_versions-additional.md?raw')
								).default,
						} as DropdownActionOption,
						{
							label: 'Monofile',
							weight: 1002,
							shouldShow: (project) =>
								project.project_type === 'resourcepack' || project.loaders.includes('datapack'),
							message: async () =>
								(
									await import('../messages/checklist-messages/versions/alternate_versions-mono.md?raw')
								).default,
						} as DropdownActionOption,
						{
							label: 'Server Files (Primary Files)',
							weight: 1002,
							shouldShow: (project) => project.project_type === 'modpack',
							message: async () =>
								(
									await import('../messages/checklist-messages/versions/alternate_versions-server.md?raw')
								).default,
						} as DropdownActionOption,
						{
							label: 'Server Files (Additional Files)',
							weight: 1002,
							suggestedStatus: 'rejected',
							severity: 'high',
							shouldShow: (project) => project.project_type === 'modpack',
							message: async () =>
								(
									await import('../messages/checklist-messages/versions/alternate_versions-server-additional.md?raw')
								).default,
						} as DropdownActionOption,
						{
							label: 'mods.zip',
							weight: 1002,
							suggestedStatus: 'rejected',
							severity: 'high',
							shouldShow: (project) => project.project_type === 'modpack',
							message: async () =>
								(
									await import('../messages/checklist-messages/versions/alternate_versions-zip.md?raw')
								).default,
						} as DropdownActionOption,
					],
				} as DropdownAction,
			],
		} as ButtonAction,
		{
			id: 'versions_incorrect_loader',
			type: 'button',
			label: 'Incorrect Loader',
			suggestedStatus: 'flagged',
			severity: 'medium',
			weight: 1001,
			message: async () =>
				(await import('../messages/checklist-messages/versions/incorrect_loader.md?raw')).default,
			enablesActions: [
				{
					id: 'versions_incorrect_loader_options',
					type: 'multi-select-chips',
					label: 'Which loader labels are incorrect?',
					joinWith: '\n',
					shouldShow: (_project, _projectV3, context) =>
						Boolean(context && getIncorrectLoaderOptions(context).length > 0),
					options: (context) => getIncorrectLoaderOptions(context),
				} as MultiSelectChipsAction,
			],
		} as ButtonAction,
		{
			id: 'versions_vanilla_assets',
			type: 'button',
			label: 'Vanilla Assets',
			suggestedStatus: `rejected`,
			severity: `medium`,
			weight: 1003,
			shouldShow: (project) => project.project_type === 'resourcepack',
			message: async () =>
				(await import('../messages/checklist-messages/versions/vanilla_assets.md?raw')).default,
		} as ButtonAction,
		{
			id: 'versions_redist_libs',
			type: 'button',
			label: 'Packed Libs',
			suggestedStatus: `rejected`,
			severity: `medium`,
			weight: 1003,
			shouldShow: (project) => project.project_type === 'mod' || project.project_type === 'plugin',
			message: async () =>
				(await import('../messages/checklist-messages/versions/redist_libs.md?raw')).default,
		} as ButtonAction,
		{
			id: 'versions_duplicate_primary_files',
			type: 'button',
			label: 'Duplicate Primary Files',
			suggestedStatus: 'flagged',
			severity: `medium`,
			weight: 1004,
			message: async () =>
				(await import('../messages/checklist-messages/versions/broken_version.md?raw')).default,
		} as ButtonAction,
		{
			id: 'unsupported_project_type',
			type: 'button',
			label: `Unsupported`,
			suggestedStatus: `rejected`,
			severity: `medium`,
			weight: 1005,
			message: async () =>
				(await import('../messages/checklist-messages/versions/unsupported_project.md?raw'))
					.default,
			relevantExtraInput: [
				{
					label: 'Project Type',
					required: true,
					variable: 'INVALID_TYPE',
				},
			],
		} as ButtonAction,
	],
}

export default versions
