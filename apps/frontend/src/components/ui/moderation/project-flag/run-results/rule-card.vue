<template>
	<div class="min-w-0 rounded-xl bg-surface-3 text-sm">
		<div class="group/header relative isolate flex min-w-0 items-center gap-1 p-3">
			<button
				type="button"
				class="absolute inset-0 cursor-pointer rounded-xl border-none bg-transparent focus-visible:outline-brand"
				:aria-label="`${rule.name ?? 'Deleted rule'} result details`"
				:aria-expanded="expanded"
				:aria-controls="detailsId"
				@click="expanded = !expanded"
			/>
			<div class="mr-auto flex min-w-0 items-start gap-1.5">
				<TagCategoryScrollTextIcon
					class="mt-0.5 size-[18px] shrink-0 text-primary transition-opacity motion-reduce:transition-none"
					:class="expanded ? 'opacity-100' : 'opacity-70 group-hover/header:opacity-100'"
					aria-hidden="true"
				/>
				<span
					class="min-w-0 break-words text-base font-medium transition-colors motion-reduce:transition-none"
					:class="expanded ? 'text-contrast' : 'text-secondary group-hover/header:text-contrast'"
				>
					{{ rule.name ?? 'Deleted rule' }}
				</span>
			</div>
			<TagItem v-if="rule.error !== null" class="shrink-0" style="--_color: var(--color-red)">
				Error
			</TagItem>
			<ButtonLink
				v-if="rule.name !== null"
				v-tooltip="'Edit rule'"
				class="relative z-10 shrink-0"
				type="quiet"
				size="sm"
				:aria-label="`Edit rule ${rule.name}`"
				:to="editTo"
			>
				<EditIcon aria-hidden="true" />
			</ButtonLink>
			<DropdownIcon
				class="pointer-events-none size-4 shrink-0 transition-[color,transform] duration-300 motion-reduce:transition-none"
				:class="
					expanded ? 'rotate-180 text-contrast' : 'text-secondary group-hover/header:text-contrast'
				"
				aria-hidden="true"
			/>
		</div>
		<Accordion
			:id="detailsId"
			:open-by-default="expanded"
			content-class="-mt-1 flex min-w-0 flex-col px-3 pb-3"
		>
			<RuleResultTabs
				v-model:value="activeView"
				:result="rule"
				:error="rule.error"
				panel-class="pt-3"
				code-class="max-h-[32rem] overflow-auto"
			/>
		</Accordion>
	</div>
</template>

<script setup lang="ts">
import type { ProjectFlagRunResult } from '@modrinth/api-client'
import { DropdownIcon, EditIcon, TagCategoryScrollTextIcon } from '@modrinth/assets'
import { Accordion, ButtonLink, type TabsValue, TagItem } from '@modrinth/ui'
import { ref, useId, watch } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import RuleResultTabs from '../rule-result-tabs.vue'

const props = defineProps<{
	rule: ProjectFlagRunResult['rules'][string]
	noFindings: boolean
	editTo: RouteLocationRaw
}>()
const detailsId = useId()
const expanded = ref(!props.noFindings)
const activeView = ref<TabsValue>('issues')
watch(
	() => props.rule.error !== null,
	(hasError) => {
		activeView.value = hasError ? 'error' : 'issues'
	},
	{ immediate: true },
)
</script>
