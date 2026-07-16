import { XIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { action, group, stage, toggle } from '../../types/node'

export default function () {
	const { projectV3: project, projectV2 } = injectProjectPageContext()

	return stage('undefined-project', 'Undefined Project')
		.hint('This project is undefined!')
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#3475ee711bf080018bf3d822a2f51a35',
		)
		.icon(XIcon)
		.navigate('/versions')
		.shown(
			computed(() => projectV2.value.versions.length === 0 && !project.value?.minecraft_server),
		)
		.children(
			group().children(
				toggle('no-versions', 'No Versions').action(action().suggestedStatus('rejected').message()),
			),
		)
}
