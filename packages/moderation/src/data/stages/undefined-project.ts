import { XIcon } from '@modrinth/assets'

import { action, button, group, md, stage } from '../../types/node'

export default stage(
	'undefined-project',
	'Undefined Project',
	'This project is undefined!',
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#3475ee711bf080018bf3d822a2f51a35',
)
	.icon(XIcon)
	.navigate('/versions')
	.shown(({ project, projectV2 }) => projectV2.versions.length === 0 && !project?.minecraft_server)
	.children(
		group().children(
			button('no_versions', 'No Versions')
				.action(
					action()
						.suggestedStatus('rejected')
						.message(md('checklist/messages/undefined-project/no_versions')),
				),
		),
	)
