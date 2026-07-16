import { VersionIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import {
	action,
	dropdown,
	group,
	md,
	option,
	stage,
	text,
	toggle,
	type MessageFn,
} from '../../types/node'

const IncorrectProjectTypeDatapackMsg: MessageFn = md(
	'checklist/messages/versions/incorrect-project-type/datapack',
)
const IncorrectProjectTypeModpackMsg: MessageFn = md(
	'checklist/messages/versions/incorrect-project-type/modpack',
)
const IncorrectProjectTypeResourcepackMsg: MessageFn = md(
	'checklist/messages/versions/incorrect-project-type/resourcepack',
)

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
				toggle('incorrect-additional-files', 'Incorrect additional files').action(
					action().suggestedStatus('flagged').severity('medium').message(),
				),

				// TODO: borked
				toggle('incorrect-project-type', 'Incorrect Project Type')
					.action(action().suggestedStatus('rejected').severity('medium'))
					.children(
						dropdown('type')
							.title('Correct Project Type')
							.none('Unknown')
							.children(
								option('modpack', 'Modpack')
									.shown(computed(() => !project.value.project_types.includes('modpack')))
									.action(action().message(IncorrectProjectTypeModpackMsg)),
								option('resourcepack', 'Resource Pack')
									.shown(computed(() => !project.value.project_types.includes('resourcepack')))
									.action(action().message(IncorrectProjectTypeResourcepackMsg)),
								option('datapack', 'Data Pack')
									.shown(computed(() => !project.value.loaders.includes('datapack')))
									.action(action().message(IncorrectProjectTypeDatapackMsg)),
							),
					),

				// TODO: borked
				toggle('alternate-versions', 'Alternate Versions')
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
								option('server-additional', 'Server Files (Additional Files)')
									.shown(computed(() => project.value.project_types.includes('modpack')))
									.action(action().message()),
								option('zip', 'mods.zip')
									.shown(computed(() => project.value.project_types.includes('modpack')))
									.action(action().message()),
							),
					),

				toggle('vanilla-assets', 'Vanilla Assets')
					.shown(computed(() => project.value.project_types.includes('resourcepack')))
					.action(action().suggestedStatus('rejected').severity('medium').message()),

				toggle('redist-libs', 'Packed Libs')
					.shown(
						computed(
							() =>
								project.value.project_types.includes('mod') ||
								project.value.project_types.includes('plugin'),
						),
					)
					.action(action().suggestedStatus('rejected').severity('medium').message()),

				toggle('duplicate-primary-files', 'Duplicate Primary Files').action(
					action().suggestedStatus('flagged').severity('medium').message(),
				),

				toggle('unsupported', 'Unsupported')
					.action(
						action()
							.suggestedStatus('rejected')
							.severity('medium')
							.message((state) => ({
								INVALID_TYPE: state['invalid-type'],
							})),
					)
					.children(text('invalid-type').title('Unsupported Type').required()),
			),
		)
}
