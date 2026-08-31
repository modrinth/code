<script setup lang="ts">
import {
	CheckIcon,
	ChevronDownIcon,
	CopyIcon,
	ExternalIcon,
	LoaderCircleIcon,
} from '@modrinth/assets'
import { ButtonLink, IconButton } from '@modrinth/ui'
import { capitalizeString, highlightCodeLines } from '@modrinth/utils'
import { computed, ref } from 'vue'

import { getSeverityBadgeColor, truncateMiddle } from './helpers'
import TechRevFlagBadges from './TechRevFlagBadges.vue'
import TechRevVerdictButtons from './TechRevVerdictButtons.vue'
import type { ClassGroup, FlattenedFileReport, TraceVerdictEvent } from './types'
import { injectTechReviewDecisions } from './use-tech-review-decisions'

const props = defineProps<{
	file: FlattenedFileReport
	classItem: ClassGroup
	expanded: boolean
	focusedDetailId?: string | null
	loadingIssues: Set<string>
	decompiledSources: Map<string, string>
}>()

const emit = defineEmits<{
	toggle: []
	verdict: [event: TraceVerdictEvent]
}>()

const { isPreReviewed, getMarkedFlagsCount } = injectTechReviewDecisions()

const showCopyFeedback = ref(false)

const truncatedPath = computed(() => truncateMiddle(props.classItem.filePath))
const pathTooltip = computed(() =>
	truncatedPath.value !== props.classItem.filePath ? props.classItem.filePath : undefined,
)

const flagDetails = computed(() => props.classItem.flags.map((flag) => flag.detail))
const allFlagsMarked = computed(
	() => getMarkedFlagsCount(props.classItem.flags) === props.classItem.flags.length,
)

const isLoadingSource = computed(() =>
	props.classItem.flags.some((flag) => props.loadingIssues.has(flag.issueId)),
)

const decompiledSource = computed(() => {
	for (const flag of props.classItem.flags) {
		const source = props.decompiledSources.get(flag.detail.id)
		if (source) return source
	}
	return undefined
})

const highlightedSource = computed(() => {
	const source = decompiledSource.value
	if (!source) return []
	return highlightCodeLines(source, 'java')
})

function emitVerdict(event: TraceVerdictEvent) {
	emit('verdict', event)
}

async function copyToClipboard() {
	const source = decompiledSource.value
	if (!source) return

	try {
		await navigator.clipboard.writeText(source)
		showCopyFeedback.value = true
		setTimeout(() => {
			showCopyFeedback.value = false
		}, 2000)
	} catch (error) {
		console.error('Failed to copy code:', error)
	}
}

function trimSlashes(path: string): string {
	if (path.startsWith('/')) {
		path = path.slice(1)
	}
	if (path.endsWith('/')) {
		path = path.slice(0, -1)
	}
	return path
}

// Creates a slicer link with a group, removes trailing and leading slashes, and if no file extension found assume .class
function createSlicerLink(url: string, group: ClassGroup | undefined) {
	const uri = new URL(url)
	const filename = uri.pathname.split('/').pop() || ''
	if (group) {
		const jarPath = trimSlashes(group.jar ? group.jar.replace('#', '/') : filename)
		const filePath = trimSlashes(
			group.filePath.startsWith('/') ? group.filePath.slice(1) : group.filePath,
		)
		const hasFileExtension = (filePath.split('/').pop() || '').includes('.')
		const file = `${jarPath}/${hasFileExtension ? filePath : `${filePath}.class`}`
		return `https://slicer.run/?url=${encodeURIComponent(url)}&file=${encodeURIComponent(file)}`
	}
	return `https://slicer.run/?url=${encodeURIComponent(url)}`
}
</script>

<template>
	<div class="flex flex-col gap-1">
		<div
			class="flex cursor-pointer items-center justify-between rounded-xl px-2 py-1 transition-colors duration-200 hover:bg-surface-3"
			@click="emit('toggle')"
		>
			<div class="flex grow items-center gap-2">
				<IconButton
					type="quiet"
					label="Toggle details"
					class="transition-transform"
					:class="{ 'rotate-180': expanded }"
				>
					<ChevronDownIcon class="h-5 w-5 text-contrast" />
				</IconButton>

				<span
					v-tooltip="pathTooltip"
					class="font-mono text-sm font-semibold"
					:class="{ 'opacity-50': allFlagsMarked }"
				>
					{{ truncatedPath }}
				</span>

				<TechRevFlagBadges :details="flagDetails" />

				<div class="ml-auto flex items-center gap-2">
					<Transition name="fade">
						<div
							v-if="isLoadingSource"
							class="rounded-full border border-solid border-surface-5 bg-surface-3 px-2.5 py-1"
						>
							<span class="flex items-center gap-1.5 text-sm font-medium text-secondary">
								<LoaderCircleIcon class="size-4 animate-spin" />
								Loading source...
							</span>
						</div>
					</Transition>
					<ButtonLink
						v-tooltip="'Open file in slicer'"
						type="outlined"
						:href="createSlicerLink(file.download_url, classItem)"
						:target="file.file_id"
						circular
						icon-only
						@click="$event.stopPropagation()"
					>
						<ExternalIcon />
					</ButtonLink>
				</div>
			</div>
		</div>

		<template v-if="expanded">
			<div
				v-for="flag in classItem.flags"
				:id="`tech-review-detail-${flag.detail.id}`"
				:key="`${flag.issueId}-${flag.detail.id}`"
				class="flex flex-col gap-2 rounded-lg border border-solid border-surface-5 bg-surface-3 py-2 pl-4"
				:class="{
					'!border-brand bg-brand-highlight': focusedDetailId === flag.detail.id,
				}"
			>
				<div class="grid grid-cols-[1fr_auto] items-center">
					<div
						class="flex items-center gap-2"
						:class="{
							'opacity-50': isPreReviewed(flag.detail.id, flag.detail.status),
						}"
					>
						<span class="text-base font-semibold text-contrast">{{
							flag.issueType.replace(/_/g, ' ')
						}}</span>
						<div
							class="rounded-full border-solid px-2.5 py-1"
							:class="getSeverityBadgeColor(flag.detail.severity)"
						>
							<span class="text-sm font-medium">{{ capitalizeString(flag.detail.severity) }}</span>
						</div>
					</div>

					<div class="me-2 flex items-center justify-end gap-2">
						<TechRevVerdictButtons
							variant="trace"
							:detail="flag.detail"
							@global-safe="emitVerdict({ detail: flag.detail, decision: 'safe', scope: 'global' })"
							@local-safe="emitVerdict({ detail: flag.detail, decision: 'safe', scope: 'local' })"
							@local-unsafe="
								emitVerdict({ detail: flag.detail, decision: 'malware', scope: 'local' })
							"
							@global-unsafe="
								emitVerdict({ detail: flag.detail, decision: 'malware', scope: 'global' })
							"
						/>
					</div>
				</div>
				<div
					v-if="flag.detail.data && Object.keys(flag.detail.data).length > 0"
					class="flex flex-wrap gap-x-4 gap-y-1 pr-4 text-sm"
				>
					<div
						v-for="[key, value] in Object.entries(flag.detail.data).sort(([a], [b]) =>
							a.localeCompare(b),
						)"
						:key="key"
						class="flex items-center gap-1.5"
					>
						<span class="text-secondary">{{ key }}:</span>
						<a
							v-if="typeof value === 'string' && value.startsWith('http')"
							:href="value"
							target="_blank"
							rel="noopener noreferrer"
							class="text-brand-blue hover:underline"
						>
							{{ value }}
						</a>
						<span v-else class="font-mono text-contrast">{{ value }}</span>
					</div>
				</div>
			</div>

			<div
				v-if="highlightedSource.length > 0"
				class="relative overflow-hidden rounded-lg border border-solid border-surface-5 bg-surface-4"
			>
				<IconButton
					v-tooltip="'Copy code'"
					type="quiet"
					label="Copy code"
					class="!absolute right-2 top-2 border-[1px]"
					@click.stop="copyToClipboard"
				>
					<CopyIcon v-if="!showCopyFeedback" />
					<CheckIcon v-else />
				</IconButton>

				<div class="overflow-x-auto bg-surface-3 py-3">
					<div
						v-for="(line, n) in highlightedSource"
						:key="n"
						class="flex font-mono text-[13px] leading-[1.6]"
					>
						<div
							class="select-none border-0 border-r border-solid border-surface-5 px-4 py-0 text-right text-primary"
							style="min-width: 3.5rem"
						>
							{{ n + 1 }}
						</div>
						<div class="flex-1 px-4 py-0 text-primary">
							<pre v-html="line || ' '"></pre>
						</div>
					</div>
				</div>
			</div>
			<div
				v-else-if="!isLoadingSource"
				class="rounded-lg border border-solid border-surface-5 bg-surface-3 p-4"
			>
				<p class="text-sm text-secondary">
					Source code not available or failed to decompile for this file.
				</p>
			</div>
		</template>
	</div>
</template>

<style scoped>
pre {
	all: unset;
	display: inline;
	white-space: pre;
}

.fade-enter-active {
	transition: opacity 0.3s ease-in;
	transition-delay: 0.2s;
}

.fade-leave-active {
	transition: opacity 0.15s ease-out;
}

.fade-enter-from,
.fade-leave-to {
	opacity: 0;
}
</style>
