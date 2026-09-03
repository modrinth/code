<template>
	<span class="inline-flex min-w-0 flex-wrap items-center gap-1">
		<template v-for="(segment, index) in decodedSegments" :key="`${segment}-${index}`">
			<ChevronRightIcon v-if="index > 0" class="size-4 shrink-0" aria-hidden="true" />
			<span
				v-tooltip="isTruncated(segment) ? segment : undefined"
				class="break-all"
				:class="{
					'font-semibold text-contrast': emphasizeLast && index === decodedSegments.length - 1,
					'text-secondary': emphasizeLast && index < decodedSegments.length - 1,
				}"
			>
				{{ formatSegment(segment) }}
			</span>
		</template>
	</span>
</template>

<script setup lang="ts">
import { ChevronRightIcon } from '@modrinth/assets'

const props = withDefaults(
	defineProps<{
		segments: readonly (string | null | undefined)[]
		truncate?: boolean
		maxLength?: number
		emphasizeLast?: boolean
		decode?: boolean
		hideBaseMrpack?: boolean
	}>(),
	{
		truncate: false,
		maxLength: 120,
		emphasizeLast: false,
		decode: true,
		hideBaseMrpack: false,
	},
)

const decodedSegments = computed(() => {
	const segments = props.segments
		.flatMap((segment) => segment?.split('#') ?? [])
		.filter((segment) => segment.length > 0)
		.map((segment) => (props.decode ? decodePath(segment) : segment))

	if (props.hideBaseMrpack && segments[0]?.toLowerCase().endsWith('.mrpack')) {
		return segments.slice(1)
	}

	return segments
})

function decodePath(path: string): string {
	try {
		return decodeURIComponent(path)
	} catch {
		return path
	}
}

function isTruncated(segment: string): boolean {
	return props.truncate && segment.length > props.maxLength
}

function formatSegment(segment: string): string {
	if (!isTruncated(segment)) return segment

	const separator = '...'
	const charsToShow = props.maxLength - separator.length
	const frontChars = Math.ceil(charsToShow / 3)
	const backChars = Math.floor((charsToShow * 2) / 3)
	return segment.slice(0, frontChars) + separator + segment.slice(-backChars)
}
</script>
