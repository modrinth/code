import { BookTextIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { check, group, md, type MessageFn, stage, toggle } from '../../types/node'
import { promptSourceRequired } from '../../utils'

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	const noSourceForkMsg: MessageFn = md('checklist/messages/license/no-source-fork')
	const noSourceMsg: MessageFn = md('checklist/messages/license/no-source')

	const needSource = promptSourceRequired(project.value.license.id, project.value.project_types)

	return stage('license', 'License')
		.hint('Is this license and link valid?')
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf080f8805df7d012a8f770',
		)
		.icon(BookTextIcon)
		.navigate('/settings/license')
		.shown(computed(() => !project.value?.minecraft_server))
		.children(
			() => (
				<div class="markdown-body w-full">
					<strong>License id:</strong> {project.value.license?.id ?? 'None'}
					<br />
					<strong>License Link:</strong>{' '}
					{project.value.license?.url ? (
						<a href={project.value.license.url} target="_blank" class="underline">
							{project.value.license.url}
						</a>
					) : (
						'None'
					)}
					{needSource && (
						<>
							<br />
							<strong>Source Link:</strong>{' '}
							{project.value.link_urls?.source?.url ? (
								<a href={project.value.link_urls.source.url} target="_blank" class="underline">
									{project.value.link_urls.source.url}
								</a>
							) : (
								'None'
							)}
						</>
					)}
				</div>
			),
			group().children(
				toggle('invalid-link', 'Invalid Link')
					.shown(computed(() => !!project.value.license?.url))
					.suggestedStatus('flagged')
					.severity('medium')
					.message()
					.children(check('custom-license', 'Invalid Link: Custom License').message())
					.collect(),

				toggle('no-source', 'No Source')
					.shown(needSource)
					.suggestedStatus('rejected')
					.severity('medium')
					.rawMessage(async (state) => {
						if (state.fork) return noSourceForkMsg(state)
						return noSourceMsg(state)
					})
					.children(check('fork', 'No Source: Fork').severity('high')),
			),
		)
}
