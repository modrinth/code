import { VersionIcon } from '@modrinth/assets'

import { button, chips, group, mdMsg, select, stage, text, toggle } from '../../types/node'

const loaderLabels: Record<string, string> = {
	neoforge: 'NeoForge',
	liteloader: 'LiteLoader',
	datapack: 'Data Pack',
	resourcepack: 'Resource Pack',
}

function formatLoaderLabel(id: string): string {
	return loaderLabels[id] ?? id
		.split(/[-_]/g)
		.map((w) => w.charAt(0).toUpperCase() + w.slice(1))
		.join(' ')
}

export default stage(
	'versions',
	'Versions',
	"Are this project's files correct?",
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e25ee711bf0804bad38e9055951ff31',
	{
		icon: VersionIcon,
		navigate: '/versions',
		shown: (_project, projectV3) => !projectV3?.minecraft_server,
	},
	[
		group().children(
			button('incorrect_additional', 'Incorrect additional files')
				.weight(1000)
				.suggestedStatus('flagged')
				.severity('medium')
				.message(mdMsg('versions/incorrect_additional_files')),

			button('incorrect_project_type', 'Incorrect Project Type')
				.weight(1001)
				.suggestedStatus('rejected')
				.severity('medium')
				.children(
					select('type', 'What type should this project be?').children(
						toggle('modpack', 'Modpack')
							.shown(({ project }) => !project.project_types.includes('modpack'))
							.weight(1001)
							.message(mdMsg('versions/invalid-modpacks')),
						toggle('resourcepack', 'Resource Pack')
							.shown(({ project }) => !project.project_types.includes('resourcepack'))
							.weight(1001)
							.message(mdMsg('versions/invalid-resourcepacks')),
						toggle('datapack', 'Data Pack')
							.shown(({ project }) => !project.loaders.includes('datapack'))
							.weight(1001)
							.message(mdMsg('versions/invalid-datapacks')),
					),
				),

			button('alternate_versions', 'Alternate Versions')
				.weight(1002)
				.suggestedStatus('rejected')
				.severity('high')
				.children(
					select('distribution', 'How are they distributed?').children(
						toggle('primary', 'Primary Files')
							.weight(1002)
							.message(mdMsg('versions/alternate_versions-primary')),
						toggle('additional', 'Additional Files')
							.weight(1002)
							.message(mdMsg('versions/alternate_versions-additional')),
						toggle('mono', 'Monofile')
							.shown(
								({ project }) =>
									project.project_types.includes('resourcepack') ||
									project.loaders.includes('datapack'),
							)
							.weight(1002)
							.message(mdMsg('versions/alternate_versions-mono')),
						toggle('server', 'Server Files (Primary Files)')
							.shown(({ project }) => project.project_types.includes('modpack'))
							.weight(1002)
							.message(mdMsg('versions/alternate_versions-server')),
						toggle('server_additional', 'Server Files (Additional Files)')
							.shown(({ project }) => project.project_types.includes('modpack'))
							.weight(1002)
							.message(mdMsg('versions/alternate_versions-server-additional')),
						toggle('zip', 'mods.zip')
							.shown(({ project }) => project.project_types.includes('modpack'))
							.weight(1002)
							.message(mdMsg('versions/alternate_versions-zip')),
					),
				),

			button('incorrect_loader', 'Incorrect Loader')
				.weight(1003)
				.suggestedStatus('flagged')
				.severity('medium')
				.message(async (ctx) => {
					const header = await mdMsg('versions/incorrect_loader')(ctx)
					const selected = ctx.state.loaders
					if (selected instanceof Set && selected.size > 0) {
						const list = [...selected].map((id) => `- ${formatLoaderLabel(id)}`).join('\n')
						return `${header}\n${list}`
					}
					return header
				})
				//TODO: different message for empty vs non empty + quick fix?
				.children(
					chips('loaders', 'Which loader labels are incorrect?').childrenFn(({ project }) =>
						project.loaders.map((id) => toggle(id, formatLoaderLabel(id)).build()),
					),
				),

			button('vanilla_assets', 'Vanilla Assets')
				.shown(({ project }) => project.project_types.includes('resourcepack'))
				.weight(1004)
				.suggestedStatus('rejected')
				.severity('medium')
				.message(mdMsg('versions/vanilla_assets')),

			button('redist_libs', 'Packed Libs')
				.shown(
					({ project }) =>
						project.project_types.includes('mod') || project.project_types.includes('plugin'),
				)
				.weight(1004)
				.suggestedStatus('rejected')
				.severity('medium')
				.message(mdMsg('versions/redist_libs')),

			button('duplicate_primary_files', 'Duplicate Primary Files')
				.weight(1005)
				.suggestedStatus('flagged')
				.severity('medium')
				.message(mdMsg('versions/broken_version')),

			button('unsupported', 'Unsupported')
				.weight(1006)
				.suggestedStatus('rejected')
				.severity('medium')
				.message(
					mdMsg('versions/unsupported_project', (ctx) => ({
						INVALID_TYPE: ctx.state.invalid_type,
					})),
				)
				.children(text('invalid_type', 'Project type').required()),
		),
	],
)
