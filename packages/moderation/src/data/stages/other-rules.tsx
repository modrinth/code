import { ListBulletedIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { group, markdown, option, stage, toggle } from '../../types/node'

//TODO: coolbot have fun :3
export default function () {
	const { projectV3: project } = injectProjectPageContext()

	return stage('rules', 'Rule Following')
		.hint('Does this project violate the rules?')
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080709084f6269835607f',
		)
		.icon(ListBulletedIcon)
		.navigate('/moderation')
		.children(
			group().children(
				toggle('paid-access-server', 'Paid access server')
					.shown(computed(() => !!project.value.minecraft_server))
					.suggestedStatus('rejected')
					.severity('critical')
					.message(),

				// TODO: chyz, the lists built by these message have empty line gaps.
				toggle('prohibited-content', 'Prohibited Content')
					.suggestedStatus('rejected')
					.severity('critical')
					.message('prohibited-content-header')
					.collect()
					.children(
						group()
							.multiSelect('options')
							.title('Which Prohibited Content rules does this project violate?')
							.children(
								option('objectionable', 'Objectionable').message(),
								option('discriminatory', 'Discriminatory or Explicit').message(),
								option('ip-infringement', 'IP Infringement').message(),
								option('legal-rights', 'Rights Violation').message(),
								option('illegal-activity', 'Illegal Activity').message(),
								option('harmful', 'Harmful or Deceptive').message(),
								option('misleading', 'Misleading claims').message(),
								option('impersonation', 'Impersonation').message(),
								option('false-endorsement', 'False Endorsement').message(),
								option('profanity', 'Profanity').message(),
								option('undisclosed-upload', 'Undisclosed Data Upload').message(),
								option('mojang-bypass', 'Mojang Bypass').message(),
							),
					),

				toggle('cheat-or-hack-advertising', 'Hacks')
					.suggestedStatus('rejected')
					.severity('critical')
					.message(),

				toggle('server-side-opt-out', 'Opt-out')
					.suggestedStatus('flagged')
					.severity('high')
					.message(),

				toggle('server-side-opt-in', 'Opt-in')
					.suggestedStatus('flagged')
					.severity('high')
					.message('server-side-opt-in-header')
					.collect()
					.children(
						group()
							.multiSelect('options')
							.title('Which features require a Server-side Opt-in?')
							.children(
								option('x-ray', 'X-ray').message(),
								option('aim-bot', 'Aim Assist').message(),
								option('movement', 'Movement').message(),
								option('pvp', 'PvP').message(),
								option('hiding-mods', 'Anti 3.x').message(),
								option('item-duplication', 'Dupe').message(),
							),
					),

				toggle('excessive-languages', 'Excessive languages')
					.shown(
						computed(
							() =>
								!!project.value.minecraft_server &&
								!!project.value.minecraft_server?.languages?.length &&
								project.value.minecraft_server.languages.length > 4,
						),
					)
					.suggestedStatus('flagged')
					.severity('low')
					.message(),

				toggle('rule-breaking-other', 'Other')
					// TODO: chyz, the required asterisk is on a separate line
					.suggestedStatus('rejected')
					.severity('critical')
					.message(undefined, (state) => ({ MESSAGE: state.message }))
					.children(
						markdown('message').title('Explain how it infringes on content rules.').required(),
					),
			),
		)
}
