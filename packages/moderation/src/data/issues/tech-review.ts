import { CodeIcon } from '@modrinth/assets'

import misusedObfuscation from '../messages/checklist/messages/tech-review/misused-obfuscation.md'
import requestSource from '../messages/checklist/messages/tech-review/request-source.md'
import requestSourceBin from '../messages/checklist/messages/tech-review/request-source-bin.md'
import requestSourceBinModpack from '../messages/checklist/messages/tech-review/request-source-bin-modpack.md'
import requestSourceObf from '../messages/checklist/messages/tech-review/request-source-obf.md'
import requestSourceObfModpack from '../messages/checklist/messages/tech-review/request-source-obf-modpack.md'
import unclearMisleading from '../messages/checklist/messages/tech-review/unclear-misleading.md'
import unsafeFunctionality from '../messages/checklist/messages/tech-review/unsafe-functionality.md'
import { issue, markdown, panel, section, select, toggle } from './component-builders/builders'
import { issueTargets } from './component-builders/targets'

const safetyChanges = [
	{
		value: 'download-consent',
		label: 'Informed consent',
		message:
			'Ensure that all file downloads, including project updates, require informed consent from the user before being initiated.',
	},
]

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
	message: ({ getSelectValues, getMarkdownValue }) => {
		const selectedChanges = getSelectValues('tech-review-safety-change-types')
		const changes = safetyChanges
			.filter(({ value }) => selectedChanges.includes(value))
			.map(({ message }) => `- ${message}`)
		const custom = getMarkdownValue('tech-review-safety-changes').trim()
		if (custom) {
			changes.push(
				/^(?:[-*+]|\d+[.)])\s/.test(custom) ? custom : `- ${custom.replaceAll('\n', '\n  ')}`,
			)
		}
		return unsafeFunctionality.replace(/-\s*$/, () => changes.join('\n'))
	},
	suggestedStatus: 'rejected',
})

export const techReviewRequestSourceIssue = issue({
	id: 'tech-review-request-source',
	title: 'Source code requested',
	category: 'Tech review',
	message: ({ projectV3, selected, getMarkdownValue, getSelectValues }) => {
		const isModpack = projectV3.project_types.includes('modpack')
		let message = requestSource
		if (selected.toggleIds.includes('tech-review-source-obfuscated')) {
			message = isModpack ? requestSourceObfModpack : requestSourceObf
		} else if (selected.toggleIds.includes('tech-review-source-binaries')) {
			message = isModpack ? requestSourceBinModpack : requestSourceBin
		}
		const files = getSelectValues('tech-review-source-external-files').map(
			(name) => `- ${(name.split(/[\\/]/).pop() ?? name).replaceAll(/[\\`*_[\]<>]/g, '\\$&')}`,
		)
		const custom = getMarkdownValue('tech-review-source-files').trim()
		if (custom) {
			files.push(
				/^(?:[-*+]|\d+[.)])\s/.test(custom) ? custom : `- ${custom.replaceAll('\n', '\n  ')}`,
			)
		}
		return message.replaceAll('- %LIST_MODS_HERE%', () => files.join('\n'))
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
		select({
			issue: techReviewUnsafeFunctionalityIssue,
			id: 'tech-review-safety-change-types',
			label: 'Required functionality changes',
			multiple: true,
			options: safetyChanges.map(({ value, label }) => ({ value, label })),
		}),
		markdown({
			issue: techReviewUnsafeFunctionalityIssue,
			id: 'tech-review-safety-changes',
			label: '',
			required: ({ getSelectValues }) =>
				getSelectValues('tech-review-safety-change-types', techReviewUnsafeFunctionalityIssue.id)
					.length === 0,
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
			select({
				issue: techReviewRequestSourceIssue,
				id: 'tech-review-source-external-files',
				label: 'Affected files',
				placeholder: 'Select affected files',
				multiple: true,
				includeSelectAllOption: true,
				options: ({ permissions, versionsV3 }) => {
					const latestVersionId = versionsV3[0]?.id
					if (!latestVersionId) return []
					const names = permissions.groups.flatMap((group) =>
						group.files
							.filter((file) => file.versions.includes(latestVersionId))
							.map((file) => file.name),
					)
					return [...new Set(names)]
						.toSorted((a, b) => a.localeCompare(b))
						.map((name) => ({ value: name, label: name.split(/[\\/]/).pop() ?? name }))
				},
			}),
			markdown({
				issue: techReviewRequestSourceIssue,
				id: 'tech-review-source-files',
				label: '',
				placeholder: 'List affected mods or files…',
				required: ({ getSelectValues }) =>
					getSelectValues('tech-review-source-external-files', techReviewRequestSourceIssue.id)
						.length === 0,
			}),
		),
	),
)
