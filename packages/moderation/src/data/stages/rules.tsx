import { ListBulletedIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { check, group, markdown, stage, toggle } from '../../types/node'

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
								check('objectionable', 'Objectionable').message(),
								check('discriminatory', 'Discriminatory or Explicit').message(),
								check('ip-infringement', 'IP Infringement').message(),
								check('legal-rights', 'Rights Violation').message(),
								check('illegal-activity', 'Illegal Activity').message(),
								check('harmful', 'Harmful or Deceptive').message(),
								check('misleading', 'Misleading claims').message(),
								check('impersonation', 'Impersonation').message(),
								check('false-endorsement', 'False Endorsement').message(),
								check('profanity', 'Profanity').message(),
								check('undisclosed-upload', 'Undisclosed Data Upload').message(),
								check('mojang-bypass', 'Mojang Bypass').message(),
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
								check('x-ray', 'X-ray').message(),
								check('aim-bot', 'Aim Assist').message(),
								check('movement', 'Movement').message(),
								check('pvp', 'PvP').message(),
								check('hiding-mods', 'Anti 3.x').message(),
								check('item-duplication', 'Dupe').message(),
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
					.suggestedStatus('rejected')
					.severity('critical')
					.message(undefined, (state) => ({ MESSAGE: state.message }))
					.children(
						markdown('message').title('Explain how it infringes on content rules.').required(),
					),
			),
		)
}
