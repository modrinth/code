<template>
	<Accordion
		:open-by-default="openByDefault"
		class="min-w-0 rounded-xl bg-surface-3 text-sm"
		button-class="group/flag-header m-0 flex w-full cursor-pointer items-center gap-1 rounded-xl border-none bg-transparent p-3 text-left text-base font-medium"
		content-class="-mt-1 flex min-w-0 flex-col gap-2 px-3 pb-3"
	>
		<template #button="{ open }">
			<span class="flex min-w-0 items-start gap-1.5">
				<TagCategoryFlagIcon
					class="mt-1 size-4 shrink-0 text-purple transition-opacity motion-reduce:transition-none"
					:class="open ? 'opacity-100' : 'opacity-70 group-hover/flag-header:opacity-100'"
					aria-hidden="true"
				/>
				<span
					class="min-w-0 break-words transition-colors motion-reduce:transition-none"
					:class="open ? 'text-contrast' : 'text-secondary group-hover/flag-header:text-contrast'"
				>
					{{ flag?.name || rule.name || (flag ? 'Flag' : 'Deleted rule') }}
				</span>
			</span>
			<DropdownIcon
				class="size-4 shrink-0 transition-[color,transform] duration-300 motion-reduce:transition-none"
				:class="
					open ? 'rotate-180 text-contrast' : 'text-secondary group-hover/flag-header:text-contrast'
				"
				aria-hidden="true"
			/>
		</template>
		<template v-if="flag">
			<p v-if="flag.reason" class="m-0 whitespace-pre-wrap break-words">
				{{ flag.reason }}
			</p>
			<span
				v-if="flag.location?.review_panel_key"
				class="inline-flex w-fit max-w-full items-center gap-1 rounded-full bg-surface-4 px-2 py-1 text-xs"
			>
				<PanelTopIcon class="size-3.5 shrink-0" aria-hidden="true" />
				<span class="min-w-0 break-words">{{ flag.location.review_panel_key }}</span>
			</span>
			<Accordion
				button-class="m-0 w-full cursor-pointer border-none bg-transparent p-0 text-left text-sm font-medium [&>div>svg]:size-4 [&>div>svg]:ml-0"
				content-class="pt-1"
			>
				<template #title>Flag data</template>
				<pre
					class="m-0 max-h-64 overflow-auto whitespace-pre-wrap break-words rounded-xl bg-surface-2"
					>{{ JSON.stringify(flag, null, 2) }}</pre
				>
			</Accordion>
		</template>
		<template v-if="showDiagnostics">
			<div v-if="rule.error !== null" class="flex min-w-0 flex-col gap-1">
				<h5 class="m-0 text-sm font-medium text-contrast">Scan error</h5>
				<p class="m-0 whitespace-pre-wrap break-words text-red">{{ rule.error }}</p>
			</div>
			<Accordion
				v-if="rule.log.length"
				open-by-default
				button-class="m-0 w-full cursor-pointer border-none bg-transparent p-0 text-left text-sm font-medium [&>div>svg]:size-4 [&>div>svg]:ml-0"
				content-class="pt-1"
			>
				<template #title>
					{{ rule.flags.length > 1 ? 'Rule logs' : 'Logs' }}
				</template>
				<pre class="m-0 whitespace-pre-wrap break-words rounded-xl bg-surface-2">{{
					rule.log.join('\n')
				}}</pre>
			</Accordion>
			<p v-if="!flag && rule.error === null && !rule.log.length" class="m-0 text-sm text-primary">
				No flags
			</p>
		</template>
	</Accordion>
</template>

<script setup lang="ts">
import type { ProjectFlag, ProjectFlagRuleResult } from '@modrinth/api-client'
import { DropdownIcon, PanelTopIcon, TagCategoryFlagIcon } from '@modrinth/assets'
import { Accordion } from '@modrinth/ui'

defineProps<{
	flag?: ProjectFlag
	rule: ProjectFlagRuleResult & { name: string | null }
	showDiagnostics: boolean
	openByDefault: boolean
}>()
</script>
