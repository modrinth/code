import { CodeIcon } from '@modrinth/assets'

import misusedObfuscation from '../messages/checklist/messages/tech-review/misused-obfuscation.md'
import requestSource from '../messages/checklist/messages/tech-review/request-source.md'
import requestSourceBin from '../messages/checklist/messages/tech-review/request-source-bin.md'
import requestSourceBinModpack from '../messages/checklist/messages/tech-review/request-source-bin-modpack.md'
import requestSourceObf from '../messages/checklist/messages/tech-review/request-source-obf.md'
import requestSourceObfModpack from '../messages/checklist/messages/tech-review/request-source-obf-modpack.md'
import unclearMisleading from '../messages/checklist/messages/tech-review/unclear-misleading.md'
import unsafeFunctionality from '../messages/checklist/messages/tech-review/unsafe-functionality.md'
import { issue, markdown, panel, section, toggle } from './component-builders/builders'
import { issueTargets } from './component-builders/targets'

export const techReviewUnclearMisleadingIssue = issue({
	id: 'tech-review-unclear-misleading',
	title: 'Unclear or misleading functionality',
	category: 'Tech review',
	message: unclearMisleading,
	actions: [issueTargets.modifyDescription()],
	suggestedStatus: 'rejected',
})

export const techReviewUnsafeFunctionalityIssue = issue({
	id: 'tech-review-unsafe-functionality',
	title: 'Unsafe functionality',
	category: 'Tech review',
	message: ({ getMarkdownValue }) =>
		unsafeFunctionality.replace(/-\s*$/, () => getMarkdownValue('tech-review-safety-changes')),
	suggestedStatus: 'rejected',
})

export const techReviewRequestSourceIssue = issue({
	id: 'tech-review-request-source',
	title: 'Source code requested',
	category: 'Tech review',
	message: ({ projectV3, selected, getMarkdownValue }) => {
		const isModpack = projectV3.project_types.includes('modpack')
		let message = requestSource
		if (selected.toggleIds.includes('tech-review-source-obfuscated')) {
			message = isModpack ? requestSourceObfModpack : requestSourceObf
		} else if (selected.toggleIds.includes('tech-review-source-binaries')) {
			message = isModpack ? requestSourceBinModpack : requestSourceBin
		}
		return message.replaceAll('- %LIST_MODS_HERE%', () =>
			getMarkdownValue('tech-review-source-files'),
		)
	},
	actions: [issueTargets.acknowledge('reply')],
	suggestedStatus: 'rejected',
})

export const techReviewMisusedObfuscationIssue = issue({
	id: 'tech-review-misused-obfuscation',
	title: 'Misused obfuscation',
	category: 'Tech review',
	message: misusedObfuscation,
	suggestedStatus: 'rejected',
})

export const techReviewPanel = panel({
	title: 'Tech review',
	hint: 'Is the project’s functionality safe and verifiable?',
	icon: CodeIcon,
	guidanceUrl: '',
}).content(
	toggle({ issue: techReviewUnclearMisleadingIssue, label: 'Unclear / Misleading' }),
	toggle({ issue: techReviewUnsafeFunctionalityIssue, label: 'Unsafe functionality' }),
	toggle({
		issue: techReviewRequestSourceIssue,
		label: 'Request source',
		clearIssueTogglesOnDisable: true,
	}),
	toggle({ issue: techReviewMisusedObfuscationIssue, label: 'Misused obfuscation' }),
	section({
		shown: ({ selected }) => selected.issueIds.includes(techReviewUnsafeFunctionalityIssue.id),
	}).content(
		markdown({
			issue: techReviewUnsafeFunctionalityIssue,
			id: 'tech-review-safety-changes',
			label: 'Required functionality changes',
			required: true,
		}),
	),
	section({
		label: 'Source request',
		shown: ({ selected }) => selected.issueIds.includes(techReviewRequestSourceIssue.id),
	}).content(
		toggle({
			issue: techReviewRequestSourceIssue,
			id: 'tech-review-source-general',
			label: 'General',
			issueCardDisplay: { label: 'General' },
			unselectToggles: ['tech-review-source-obfuscated', 'tech-review-source-binaries'],
		}),
		toggle({
			issue: techReviewRequestSourceIssue,
			id: 'tech-review-source-obfuscated',
			label: 'Obfuscated code',
			issueCardDisplay: { label: 'Obfuscated code' },
			unselectToggles: ['tech-review-source-general', 'tech-review-source-binaries'],
		}),
		toggle({
			issue: techReviewRequestSourceIssue,
			id: 'tech-review-source-binaries',
			label: 'Native code / binaries',
			issueCardDisplay: { label: 'Native code / binaries' },
			unselectToggles: ['tech-review-source-general', 'tech-review-source-obfuscated'],
		}),
		section({
			shown: ({ projectV3, selected }) =>
				projectV3.project_types.includes('modpack') &&
				(selected.toggleIds.includes('tech-review-source-obfuscated') ||
					selected.toggleIds.includes('tech-review-source-binaries')),
		}).content(
			markdown({
				issue: techReviewRequestSourceIssue,
				id: 'tech-review-source-files',
				label: 'Affected mods or files',
				required: true,
			}),
		),
	),
)
