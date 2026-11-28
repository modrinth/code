<template>
	<TagItem v-if="run.num_errors > 0" v-tooltip="errorLabel" style="--_color: var(--color-red)">
		<XCircleIcon aria-hidden="true" />
		{{ errorCount }}<span class="sr-only"> {{ errorNoun }}</span>
	</TagItem>
	<TagItem
		v-tooltip="
			uniqueFlagKeyCount === undefined
				? `${flaggedCount} flagged`
				: flagCountTooltip(run.num_flagged, uniqueFlagKeyCount)
		"
		style="--_color: var(--color-purple)"
	>
		<TagCategoryFlagIcon aria-hidden="true" />
		{{ flaggedCount }}<span class="sr-only"> flagged</span>
	</TagItem>
	<TagItem v-tooltip="`${approvableCount} would be approved`" style="--_color: var(--color-green)">
		<BadgeCheckIcon aria-hidden="true" />
		{{ approvableCount }}<span class="sr-only"> would be approved</span>
	</TagItem>
</template>

<script setup lang="ts">
import type { ProjectFlagRun } from '@modrinth/api-client'
import { BadgeCheckIcon, TagCategoryFlagIcon, XCircleIcon } from '@modrinth/assets'
import { TagItem } from '@modrinth/ui'

import { flagCountTooltip } from './run-format'

const props = defineProps<{
	run: Pick<ProjectFlagRun, 'num_errors' | 'num_flagged' | 'num_no_flags'>
	uniqueFlagKeyCount?: number
}>()

const errorCount = computed(() => props.run.num_errors.toLocaleString('en-US'))
const errorNoun = computed(() => (props.run.num_errors === 1 ? 'scan error' : 'scan errors'))
const errorLabel = computed(() => `${errorCount.value} ${errorNoun.value}`)
const flaggedCount = computed(() => props.run.num_flagged.toLocaleString('en-US'))
const approvableCount = computed(() => props.run.num_no_flags.toLocaleString('en-US'))
</script>
