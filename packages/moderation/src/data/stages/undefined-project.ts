import { XIcon } from '@modrinth/assets'

import { action, toggle, group, md, stage, stageFn } from '../../types/node'

export default stageFn((project, projectV2) => stage('undefined-project', 'Undefined Project')
	.hint('This project is undefined!')
	.guidance('https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#3475ee711bf080018bf3d822a2f51a35')
	.icon(XIcon)
	.navigate('/versions')
	.shown(projectV2.versions.length === 0 && !project?.minecraft_server)
	.children(
		group().children(
			toggle('no_versions', 'No Versions')
				.action(
					action()
						.suggestedStatus('rejected')
						.message(md('checklist/messages/undefined-project/no_versions')),
				),
		),
	),
)
