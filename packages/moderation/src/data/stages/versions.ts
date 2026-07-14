import { VersionIcon } from '@modrinth/assets'

import { action, toggle, dropdown, group, md, option, stage, stageFn, text } from '../../types/node'

const loaderLabels: Record<string, string> = {
	neoforge: 'NeoForge',
	liteloader: 'LiteLoader',
	datapack: 'Data Pack',
	resourcepack: 'Resource Pack',
}

function formatLoaderLabel(id: string): string {
	return (
		loaderLabels[id] ??
		id
			.split(/[-_]/g)
			.map((w) => w.charAt(0).toUpperCase() + w.slice(1))
			.join(' ')
	)
}

export default stageFn((project) =>
	stage('versions', 'Versions')
		.hint("Are this project's files correct?")
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e25ee711bf0804bad38e9055951ff31',
		)
		.icon(VersionIcon)
		.navigate('/versions')
		.shown(!project?.minecraft_server)
		.children(
			group().children(
				toggle('incorrect_additional_files', 'Incorrect additional files').action(
					action()
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/versions/incorrect_additional_files')),
				),

				toggle('incorrect_project_type', 'Incorrect Project Type')
					.action(action().suggestedStatus('rejected').severity('medium'))
					.children(
						dropdown('type')
              .title("Correct Project Type")
							.none('Unknown')
							.children(
								option('modpack', 'Modpack')
									.shown(!project.project_types.includes('modpack'))
									.action(action().message(md('checklist/messages/versions/invalid-modpacks'))),
								option('resourcepack', 'Resource Pack')
									.shown(!project.project_types.includes('resourcepack'))
									.action(
										action().message(md('checklist/messages/versions/invalid-resourcepacks')),
									),
								option('datapack', 'Data Pack')
									.shown(!project.loaders.includes('datapack'))
									.action(action().message(md('checklist/messages/versions/invalid-datapacks'))),
							),
					),

				toggle('alternate_versions', 'Alternate Versions')
					.action(action().suggestedStatus('rejected').severity('high'))
					.children(
						dropdown('distribution')
							.title('Distribution Type')
							.none('Unknown')
							.children(
								option('primary', 'Primary Files').action(
									action().message(md('checklist/messages/versions/alternate_versions-primary')),
								),
								option('additional', 'Additional Files').action(
									action().message(md('checklist/messages/versions/alternate_versions-additional')),
								),
								option('mono', 'Monofile')
									.shown(
										project.project_types.includes('resourcepack') ||
											project.loaders.includes('datapack'),
									)
									.action(
										action().message(md('checklist/messages/versions/alternate_versions-mono')),
									),
								option('server', 'Server Files (Primary Files)')
									.shown(project.project_types.includes('modpack'))
									.action(
										action().message(md('checklist/messages/versions/alternate_versions-server')),
									),
								option('server_additional', 'Server Files (Additional Files)')
									.shown(project.project_types.includes('modpack'))
									.action(
										action().message(
											md('checklist/messages/versions/alternate_versions-server-additional'),
										),
									),
								option('zip', 'mods.zip')
									.shown(project.project_types.includes('modpack'))
									.action(
										action().message(md('checklist/messages/versions/alternate_versions-zip')),
									),
							),
					),

				toggle('incorrect_loader', 'Incorrect Loader')
					.action(
						action()
							.suggestedStatus('flagged')
							.severity('medium')
							.message(async (ctx) => {
								const header = await md('checklist/messages/versions/incorrect_loader')(ctx)
								const selected = ctx.state.loaders
								if (selected instanceof Set && selected.size > 0) {
									const list = [...selected].map((id) => `- ${formatLoaderLabel(id)}`).join('\n')
									return `${header}\n${list}`
								}
								return header
							}),
					)
					//TODO: different message for empty vs non empty + quick fix?
					.children(
						group('loaders')
							.title('Incorrect Loaders')
							.multiSelect()
							.children(...project.loaders.map((id) => option(id, formatLoaderLabel(id)))),
					),

				toggle('vanilla_assets', 'Vanilla Assets')
					.shown(project.project_types.includes('resourcepack'))
					.action(
						action()
							.suggestedStatus('rejected')
							.severity('medium')
							.message(md('checklist/messages/versions/vanilla_assets')),
					),

				toggle('redist_libs', 'Packed Libs')
					.shown(project.project_types.includes('mod') || project.project_types.includes('plugin'))
					.action(
						action()
							.suggestedStatus('rejected')
							.severity('medium')
							.message(md('checklist/messages/versions/redist_libs')),
					),

				toggle('duplicate_primary_files', 'Duplicate Primary Files').action(
					action()
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/versions/broken_version')),
				),

				toggle('unsupported', 'Unsupported')
					.action(
						action()
							.suggestedStatus('rejected')
							.severity('medium')
							.message(
								md('checklist/messages/versions/unsupported_project', (ctx) => ({
									INVALID_TYPE: ctx.state.invalid_type,
								})),
							),
					)
					.children(text('invalid_type')
            .title('Unsupported Type')
            .required()),
			),
		),
)
