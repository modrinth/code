import type { Labrinth } from '@modrinth/api-client'
import { createContext } from '@modrinth/ui'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { computed, provide, ref } from 'vue'

import type { ProjectReviewPageContext } from '~/providers/project-review'

import { getHighestSeverity } from '../helpers'
import type { FlattenedFileReport } from '../types'
import { TECH_REVIEW_DECISIONS_KEY, useTechReviewDecisions } from '../use-tech-review-decisions'
import FileItem from './item.vue'

type Detail = Labrinth.TechReview.Internal.ReportIssueDetail

const [, provideFileItemProject] =
	createContext<Pick<ProjectReviewPageContext, 'project'>>('ProjectReviewPage')

const project: Labrinth.Projects.v3.Project = {
	id: 'storybook-tech-review',
	slug: 'web-displays-together',
	project_types: ['mod'],
	games: ['minecraft:java-edition'],
	team_id: 'storybook-team',
	name: 'Web Displays Together',
	summary: 'A project for previewing archive review states.',
	description: '',
	published: '2026-11-01T00:00:00Z',
	updated: '2026-11-02T00:00:00Z',
	status: 'processing',
	license: { id: 'MIT', name: 'MIT License' },
	downloads: 0,
	followers: 0,
	categories: [],
	additional_categories: [],
	loaders: ['forge'],
	mrpack_loaders: [],
	versions: ['storybook-version'],
	link_urls: {},
	gallery: [],
	thread_id: 'storybook-thread',
	monetization_status: 'monetized',
	side_types_migration_review_status: 'reviewed',
	environment: ['unknown'],
}

function traces(count: number, overrides: Partial<Detail> = {}): Detail[] {
	return Array.from({ length: count }, (_, index) => ({
		id: `${overrides.global_status ?? 'local'}-${overrides.status ?? 'pending'}-${overrides.severity ?? 'severe'}-${index}`,
		issue_id: 'storybook-issue',
		key: `storybook-trace-${index}`,
		jar: null,
		file_path: `com/example/Example${index}.class`,
		decompiled_source: null,
		data: {},
		severity: 'severe',
		local_status: null,
		global_status: null,
		status: 'pending',
		...overrides,
	}))
}

function archive(
	details: Detail[],
	overrides: Partial<FlattenedFileReport> = {},
): FlattenedFileReport {
	return {
		id: 'storybook-report',
		report_id: 'storybook-report',
		file_id: 'storybook-file',
		version_id: 'storybook-version',
		version_number: '1.05',
		created: '2026-11-02T00:00:00Z',
		flag_reason: 'delphi',
		severity: getHighestSeverity(details),
		file_name: 'webdisplaystogether-1.05.jar',
		file_size: 285223158,
		download_url: 'https://example.com/webdisplaystogether-1.05.jar',
		issues: details.length
			? [
					{
						id: 'storybook-issue',
						report_id: 'storybook-report',
						issue_type: 'suspicious_code',
						details,
					},
				]
			: [],
		...overrides,
	}
}

const globallyPassed = (count: number) => traces(count, { status: 'safe', global_status: 'safe' })
const locallyPassed = (count: number) => traces(count, { status: 'safe', local_status: 'safe' })
const unsafe = (count: number) => traces(count, { status: 'unsafe', local_status: 'unsafe' })

const partialReview = archive([...globallyPassed(236), ...locallyPassed(43), ...traces(20)])

function fileItemPreview(
	args: { file: FlattenedFileReport; selected?: boolean },
	missingProject = false,
) {
	return {
		components: { FileItem },
		setup() {
			provideFileItemProject({ project: ref(missingProject ? undefined : project) })
			provide(TECH_REVIEW_DECISIONS_KEY, useTechReviewDecisions(computed(() => [args.file])))
			return { args }
		},
		template: /* html */ `
			<div class="w-[34rem] max-w-[calc(100vw-2rem)] rounded-xl border border-solid border-surface-5 bg-surface-2 p-1.5" data-file-item-preview>
				<FileItem v-bind="args" />
			</div>
		`,
	}
}

const meta = {
	title: 'Website/Moderation/TechReview/FileItem',
	component: FileItem,
	parameters: { layout: 'centered' },
	args: { file: partialReview, selected: false },
	argTypes: {
		file: { control: false },
		onViewFlags: { action: 'viewFlags' },
	},
	render: (args, context) => fileItemPreview(args, context.parameters.missingProject),
} satisfies Meta<typeof FileItem>

export default meta
type Story = StoryObj<typeof meta>

export const PartiallyReviewed: Story = {}
export const Selected: Story = { args: { selected: true } }
export const Unreviewed: Story = { args: { file: archive(traces(20)) } }
export const PendingMalware: Story = {
	args: { file: archive(traces(3, { severity: 'malware' })) },
}
export const PendingHigh: Story = {
	args: { file: archive(traces(3, { severity: 'high' })) },
}
export const PendingMedium: Story = {
	args: { file: archive(traces(3, { severity: 'medium' })) },
}
export const PendingLow: Story = {
	args: { file: archive(traces(3, { severity: 'low' })) },
}
export const HiddenOnly: Story = {
	args: { file: archive(traces(3, { severity: 'hidden' })) },
}
export const OneHiddenTrace: Story = {
	args: {
		file: archive([...globallyPassed(236), ...traces(19), ...traces(1, { severity: 'hidden' })]),
	},
}
export const MultipleHiddenTraces: Story = {
	args: {
		file: archive([...globallyPassed(236), ...traces(17), ...traces(3, { severity: 'hidden' })]),
	},
}
export const AllGloballyPassed: Story = {
	args: { file: archive(globallyPassed(299)) },
}
export const ReviewedSafe: Story = {
	args: { file: archive([...globallyPassed(236), ...locallyPassed(63)]) },
}
export const ReviewedUnsafe: Story = {
	args: { file: archive([...globallyPassed(236), ...locallyPassed(62), ...unsafe(1)]) },
}
export const UnsafeWithPending: Story = {
	args: {
		file: archive([...globallyPassed(236), ...locallyPassed(42), ...unsafe(1), ...traces(20)]),
	},
}
export const GloballyUnsafe: Story = {
	args: { file: archive(traces(1, { status: 'unsafe', global_status: 'unsafe' })) },
}
export const GloballyPassedSeverityExcluded: Story = {
	args: { file: archive([...globallyPassed(236), ...traces(3, { severity: 'low' })]) },
}
export const NoFlags: Story = { args: { file: archive([]) } }
export const ManualReview: Story = {
	args: {
		file: archive([], { flag_reason: 'manual' as FlattenedFileReport['flag_reason'] }),
	},
}
export const MissingProject: Story = { parameters: { missingProject: true } }
export const LongFilename: Story = {
	args: {
		file: archive([...unsafe(1), ...traces(3)], {
			file_name: 'webdisplaystogether-forge-1.20.1-1.05-release-with-bundled-dependencies.jar',
		}),
	},
}

export const AllStates: Story = {
	parameters: { layout: 'fullscreen', controls: { disable: true } },
	render: () => ({
		setup() {
			const states = Object.entries({
				PartiallyReviewed,
				Selected,
				Unreviewed,
				PendingMalware,
				PendingHigh,
				PendingMedium,
				PendingLow,
				HiddenOnly,
				OneHiddenTrace,
				MultipleHiddenTraces,
				AllGloballyPassed,
				ReviewedSafe,
				ReviewedUnsafe,
				UnsafeWithPending,
				GloballyUnsafe,
				GloballyPassedSeverityExcluded,
				NoFlags,
				ManualReview,
				MissingProject,
				LongFilename,
			}).map(([name, story]) => ({
				name: name.replace(/([a-z])([A-Z])/g, '$1 $2'),
				component: fileItemPreview(
					{
						file: story.args?.file ?? partialReview,
						selected: story.args?.selected ?? false,
					},
					story.parameters?.missingProject,
				),
			}))
			return { states }
		},
		template: /* html */ `
			<div class="grid w-fit grid-cols-1 gap-4 p-4" data-file-item-gallery>
				<section v-for="state in states" :key="state.name" class="min-w-0">
					<h2 class="m-0 mb-2 text-sm font-medium text-contrast">{{ state.name }}</h2>
					<component :is="state.component" />
				</section>
			</div>
		`,
	}),
}
