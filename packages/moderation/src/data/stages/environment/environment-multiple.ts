import { GlobeIcon } from '@modrinth/assets'

import { button, group, mdMsg, mdText, prose, stage } from '../../../types/node'

export default stage(
	'environment',
	'Environment',
	"Is the project's environment information accurate?",
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e25ee711bf0802d9a9bdb82dce040eb',
	{
		icon: GlobeIcon,
		navigate: '/settings/versions',
		shown: (_project, projectV3) =>
			(projectV3?.environment?.length ?? 0) !== 1 && !projectV3?.minecraft_server,
	},
	[
		prose(mdText('environment/environment-multiple')),

		group().children(
			button('side_types_inaccurate', 'Inaccurate')
				.shown(({ project }) =>
					project.project_types.includes('mod') || project.project_types.includes('modpack'),
				)
				.weight(800)
				.suggestedStatus('flagged')
				.severity('low')
				.message(mdMsg('environment/inaccurate')),
		),
	],
)
