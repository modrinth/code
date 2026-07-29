import { VersionIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { dropdown, group, option, stage, text, toggle } from '../../types/node'

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
					.message(),

				toggle('incorrect-project-type', 'Incorrect Project Type')
					.suggestedStatus('rejected')
					.children(
						dropdown('type')
							.title('Correct Project Type')
              .required()
							.none('Unknown')
							.options(
								option('modpack', 'Modpack').message(),
								option('resourcepack', 'Resource Pack').message(),
								option('datapack', 'Data Pack').message(),
							),
					)
					.collect(),

				toggle('alternate-versions', 'Alternate Versions')
					.suggestedStatus('rejected')
					.children(
						dropdown('distribution')
							.title('Distribution Type')
              .required()
							.none('Unknown')
							.options(
								option('primary', 'Primary Files').message(),
								option('additional', 'Additional Files').message(),
								option('mono', 'Monofile')
									.shown(
										computed(
											() =>
												project.value.project_types.includes('resourcepack') ||
												project.value.loaders.includes('datapack'),
										),
									)
									.message(),
								option('server', 'Server Files (Primary Files)')
									.shown(computed(() => project.value.project_types.includes('modpack')))
									.message(),
								option('server-additional', 'Server Files (Additional Files)')
									.shown(computed(() => project.value.project_types.includes('modpack')))
									.message(),
								option('zip', 'mods.zip')
									.shown(computed(() => project.value.project_types.includes('modpack')))
									.message(),
							),
					)
					.collect(),

				toggle('vanilla-assets', 'Vanilla Assets')
					.shown(computed(() => project.value.project_types.includes('resourcepack')))
					.suggestedStatus('rejected')
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
					.message(),

				toggle('duplicate-primary-files', 'Duplicate Primary Files')
					.suggestedStatus('flagged')
					.message(),

				toggle('unsupported', 'Unsupported')
					.suggestedStatus('rejected')
					.message((state) => ({
						INVALID_TYPE: state['invalid-type'],
					}))
					.children(text('invalid-type').title('Unsupported Type').required()),
			),
		)
}
