import { VersionIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { action, dropdown, group, option, stage, text, toggle } from '../../types/node'

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	return stage('versions', 'Versions')
		.hint("Are this project's files correct?")
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e25ee711bf0804bad38e9055951ff31',
		)
		.icon(VersionIcon)
		.navigate('/versions')
		.shown(computed(() => !project.value?.minecraft_server))
		.children(
			group().children(
				toggle('incorrect_additional_files', 'Incorrect additional files').action(
					action().suggestedStatus('flagged').severity('medium').message(),
				),

				toggle('incorrect_project_type', 'Incorrect Project Type')
					.action(action().suggestedStatus('rejected').severity('medium'))
					.children(
						dropdown('type')
							.title('Correct Project Type')
							.none('Unknown')
							.children(
								option('modpack', 'Modpack')
									.shown(computed(() => !project.value.project_types.includes('modpack')))
									.action(action().message()),
								option('resourcepack', 'Resource Pack')
									.shown(computed(() => !project.value.project_types.includes('resourcepack')))
									.action(action().message()),
								option('datapack', 'Data Pack')
									.shown(computed(() => !project.value.loaders.includes('datapack')))
									.action(action().message()),
							),
					),

				toggle('alternate_versions', 'Alternate Versions')
					.action(action().suggestedStatus('rejected').severity('high'))
					.children(
						dropdown('distribution')
							.title('Distribution Type')
							.none('Unknown')
							.children(
								option('primary', 'Primary Files').action(action().message()),
								option('additional', 'Additional Files').action(action().message()),
								option('mono', 'Monofile')
									.shown(
										computed(
											() =>
												project.value.project_types.includes('resourcepack') ||
												project.value.loaders.includes('datapack'),
										),
									)
									.action(action().message()),
								option('server', 'Server Files (Primary Files)')
									.shown(computed(() => project.value.project_types.includes('modpack')))
									.action(action().message()),
								option('server_additional', 'Server Files (Additional Files)')
									.shown(computed(() => project.value.project_types.includes('modpack')))
									.action(action().message()),
								option('zip', 'mods.zip')
									.shown(computed(() => project.value.project_types.includes('modpack')))
									.action(action().message()),
							),
					),

				toggle('vanilla_assets', 'Vanilla Assets')
					.shown(computed(() => project.value.project_types.includes('resourcepack')))
					.action(action().suggestedStatus('rejected').severity('medium').message()),

				toggle('redist_libs', 'Packed Libs')
					.shown(
						computed(
							() =>
								project.value.project_types.includes('mod') ||
								project.value.project_types.includes('plugin'),
						),
					)
					.action(action().suggestedStatus('rejected').severity('medium').message()),

				toggle('duplicate_primary_files', 'Duplicate Primary Files').action(
					action().suggestedStatus('flagged').severity('medium').message(),
				),

				toggle('unsupported', 'Unsupported')
					.action(
						action()
							.suggestedStatus('rejected')
							.severity('medium')
							.message((state) => ({
								INVALID_TYPE: state.invalid_type,
							})),
					)
					.children(text('invalid_type').title('Unsupported Type').required()),
			),
		)
}
