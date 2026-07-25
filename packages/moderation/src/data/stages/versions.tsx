import { VersionIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { check, dropdown, group, stage, text, toggle } from '../../types/node'

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
				toggle('incorrect-additional-files', 'Incorrect additional files')
					.suggestedStatus('flagged')
					.severity('medium')
					.message(),

				toggle('incorrect-project-type', 'Incorrect Project Type')
					.suggestedStatus('rejected')
					.severity('medium')
					.children(
						dropdown('type')
							.title('Correct Project Type')
							.none('Unknown')
							.children(
								check('modpack', 'Modpack')
									.shown(computed(() => !project.value.project_types.includes('modpack')))
									.message(),
								check('resourcepack', 'Resource Pack')
									.shown(computed(() => !project.value.project_types.includes('resourcepack')))
									.message(),
								check('datapack', 'Data Pack')
									.shown(computed(() => !project.value.loaders.includes('datapack')))
									.message(),
							),
					)
					.collect(),

				toggle('alternate-versions', 'Alternate Versions')
					.suggestedStatus('rejected')
					.severity('high')
					.children(
						dropdown('distribution')
							.title('Distribution Type')
							.none('Unknown')
							.children(
								check('primary', 'Primary Files').message(),
								check('additional', 'Additional Files').message(),
								check('mono', 'Monofile')
									.shown(
										computed(
											() =>
												project.value.project_types.includes('resourcepack') ||
												project.value.loaders.includes('datapack'),
										),
									)
									.message(),
								check('server', 'Server Files (Primary Files)')
									.shown(computed(() => project.value.project_types.includes('modpack')))
									.message(),
								check('server-additional', 'Server Files (Additional Files)')
									.shown(computed(() => project.value.project_types.includes('modpack')))
									.message(),
								check('zip', 'mods.zip')
									.shown(computed(() => project.value.project_types.includes('modpack')))
									.message(),
							),
					)
					.collect(),

				toggle('vanilla-assets', 'Vanilla Assets')
					.shown(computed(() => project.value.project_types.includes('resourcepack')))
					.suggestedStatus('rejected')
					.severity('medium')
					.message(),

				toggle('redist-libs', 'Packed Libs')
					.shown(
						computed(
							() =>
								project.value.project_types.includes('mod') ||
								project.value.project_types.includes('plugin'),
						),
					)
					.suggestedStatus('rejected')
					.severity('medium')
					.message(),

				toggle('duplicate-primary-files', 'Duplicate Primary Files')
					.suggestedStatus('flagged')
					.severity('medium')
					.message(),

				toggle('unsupported', 'Unsupported')
					.suggestedStatus('rejected')
					.severity('medium')
					.message(undefined, (state) => ({
						INVALID_TYPE: state['invalid-type'],
					}))
					.children(text('invalid-type').title('Unsupported Type').required()),
			),
		)
}
