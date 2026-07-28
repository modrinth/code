import { ScaleIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { group, stage, text, toggle } from '../../types/node'
import {Priorities} from "../priorities.ts";

//TODO chyz
//TODO coolbot needs discussion
export default function () {
	const { projectV3: project } = injectProjectPageContext()

	return stage('post-approval', 'Post-Approval')
		.hint('Issue warnings, notices, or takedowns?')
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#3475ee711bf080c5a13cda0b1e4ae9ed',
		)
		.icon(ScaleIcon)
		.shown(computed(() => project.value.status === 'approved'))
		.children(
			group().children(
				toggle('issue-warning', 'Issue warning')
					.suggestedStatus('approved')
					.message()
          .priority(Priorities.tempLast),

				toggle('missed-deadline', 'Missed due date')
					.suggestedStatus('flagged')
					.message((state) => ({
						STATUS: state.status,
					}))
          .priority(Priorities.tempLast)
					.children(
						//TODO: chyz this shouldn't need to be provided by moderator
						text('status').title('What status is the project being set to?').required(),
					),

				toggle('metadata-issue', 'Incorrect metadata')
					.suggestedStatus('approved')
					.message()
					.children(
						toggle('dependencies', 'Missing Dependencies')
							.message((state) => ({
								DEPENDENCY_NAME: state['name'],
								DEPENDENCY_LINK: state['link'],
							}))
							.children(
								text('name').title('Dependency name').required(),
								text('link').title('Dependency link').required(),
							),

						toggle('mc-versions', 'Game versions')
							.message((state) => ({
								SPECIFICS: state.specifics,
							}))
							.children(text('specifics').title('More details about the game versions issue?')),

						toggle('loaders', 'Loaders')
							.message((state) => ({
								SPECIFICS: state.specifics,
							}))
							.children(text('specifics').title('More details about the loaders issue?')),

						toggle('license', 'Inconsistent Licensing').message(),
					)
					.collect(),
			),
		)
}
