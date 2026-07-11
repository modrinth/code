import { GlobeIcon } from '@modrinth/assets'

import { action, button, group, label, md, stage } from '../../types/node'

const textSingle = md('checklist/text/environment/environment')
const textMultiple = md('checklist/text/environment/environment-multiple')

export default stage(
	'environment',
	'Environment',
	"Is the project's environment information accurate?",
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e25ee711bf0802d9a9bdb82dce040eb',
)
	.icon(GlobeIcon)
	.navigate('/settings/versions')
	.shown(({ project }) => !project?.minecraft_server)
	.children(
		label((ctx) =>
			(ctx.project.environment?.length ?? 0) === 1
				? textSingle(ctx)
				: textMultiple(ctx),
		),

		group().children(
			button('side_types_inaccurate', 'Inaccurate')
				.shown(({ project }) =>
					project.project_types.includes('mod') || project.project_types.includes('modpack'),
				)
				.action(
					action()
						.weight(800)
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/environment/inaccurate')),
				),
		),
	)
