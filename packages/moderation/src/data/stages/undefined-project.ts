import { XIcon } from '@modrinth/assets'

import { button, group, mdMsg, stage } from '../../types/node'

export default stage(
	'undefined-project',
	'Undefined Project',
	'This project is undefined!',
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#3475ee711bf080018bf3d822a2f51a35',
	{
		icon: XIcon,
		navigate: '/versions',
		shown: (project, projectV3) => project.versions.length === 0 && !projectV3?.minecraft_server,
	},
	[
		group().children(
			button('no_versions', 'No Versions')
				.weight(-100)
				.suggestedStatus('rejected')
				.message(mdMsg('undefined-project/no_versions')),
		),
	],
)
