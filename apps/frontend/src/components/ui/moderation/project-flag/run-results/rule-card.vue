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
				<TagCategoryFlagIcon
					class="mt-1 size-4 shrink-0 text-purple transition-opacity motion-reduce:transition-none"
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
			<TagItem v-if="rule.flags.length > 0" class="shrink-0" style="--_color: var(--color-purple)">
				{{ rule.flags.length }} {{ rule.flags.length === 1 ? 'flag' : 'flags' }}
			</TagItem>
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
			content-class="-mt-1 flex min-w-0 flex-col gap-2 px-3 pb-3"
		>
			<p v-if="rule.error !== null" class="m-0 whitespace-pre-wrap break-words text-red">
				{{ rule.error }}
			</p>
			<p v-else-if="noFindings" class="m-0 text-primary">No flags</p>
			<FlagCard
				v-for="(flag, index) in rule.flags"
				:key="index"
				:flag="flag"
				:class="{ 'border-0 border-t border-solid border-divider pt-2': index > 0 }"
			/>
		</Accordion>
	</div>
</template>

<script setup lang="ts">
import type { ProjectFlagRunResult } from '@modrinth/api-client'
import { DropdownIcon, EditIcon, TagCategoryFlagIcon } from '@modrinth/assets'
import { Accordion, ButtonLink, TagItem } from '@modrinth/ui'
import { ref, useId } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import FlagCard from './flag-card.vue'

const props = defineProps<{
	rule: ProjectFlagRunResult['rules'][string]
	noFindings: boolean
	editTo: RouteLocationRaw
}>()
const detailsId = useId()
const expanded = ref(!props.noFindings)
</script>
