<template>
	<div class="flex min-h-0 min-w-0 flex-col">
		<Tabs
			v-model:value="activeView"
			:tabs="tabs"
			wrap
			color="gray"
			aria-label="Rule result views"
			class="!h-auto !w-full shrink-0 !rounded-none !border-x-0 !border-t-0 !border-b-surface-4 !px-2 !py-0 !shadow-none [&_[role=tab][aria-selected=true]]:!border-b-green [&_[role=tab]]:!rounded-none [&_[role=tab]]:!border-0 [&_[role=tab]]:!border-b-2 [&_[role=tab]]:!border-transparent [&_[role=tab]]:!bg-transparent [&_[role=tab]]:py-2"
		/>
		<div
			role="tabpanel"
			:aria-label="tabs.find((tab) => tab.value === activeView)?.label"
			:aria-busy="running"
			tabindex="0"
			class="min-h-0 min-w-0"
			:class="panelClass"
		>
			<p v-if="running" role="status" class="m-0 text-sm text-secondary">Running tests…</p>
			<pre
				v-else-if="activeView === 'error' && error != null"
				role="alert"
				class="m-0 whitespace-pre-wrap break-words font-mono text-sm text-red"
				v-text="error"
			/>
			<p v-else-if="!result" class="m-0 text-sm text-secondary">
				{{ error ? 'Test failed.' : 'Run all to see this project’s results.' }}
			</p>
			<pre
				v-else-if="activeView === 'raw' || (activeView === 'log' && result.log.length)"
				class="m-0 whitespace-pre-wrap break-words font-mono"
				:class="[activeView === 'raw' ? 'bg-surface-2 text-xs' : 'text-sm', codeClass]"
				v-text="activeView === 'raw' ? JSON.stringify(result, null, 2) : result.log.join('\n')"
			/>
			<p v-else-if="activeView === 'log'" class="m-0 text-sm text-secondary">No print output.</p>
			<p v-else-if="!result.flags.length" class="m-0 text-sm text-secondary">
				{{ result.error ? 'Test failed.' : 'No flags found.' }}
			</p>
			<ul v-else class="m-0 flex list-none flex-col gap-2 p-0">
				<li
					v-for="(flag, index) in result.flags"
					:key="index"
					class="flex min-w-0 flex-col gap-1 text-sm"
				>
					<div class="flex min-w-0 items-start gap-1.5 text-purple">
						<TagCategoryFlagIcon class="mt-0.5 size-4 shrink-0" aria-hidden="true" />
						<code class="break-all font-mono">{{ flag.key }}</code>
					</div>
					<span v-if="flag.reason" class="min-w-0 whitespace-pre-wrap break-words">{{
						flag.reason
					}}</span>
				</li>
			</ul>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { ProjectFlagRuleResult } from '@modrinth/api-client'
import { TagCategoryFlagIcon } from '@modrinth/assets'
import { Tabs, type TabsValue } from '@modrinth/ui'
import { computed } from 'vue'

const props = defineProps<{
	result?: ProjectFlagRuleResult
	error?: string | null
	running?: boolean
	panelClass?: string
	codeClass?: string
}>()
const activeView = defineModel<TabsValue>('value', { required: true })
const tabs = computed(() => [
	{ value: 'issues', label: props.result ? `Flags (${props.result.flags.length})` : 'Flags' },
	{ value: 'log', label: props.result ? `Log (${props.result.log.length})` : 'Log' },
	...(props.error != null ? [{ value: 'error', label: 'Error' }] : []),
	{ value: 'raw', label: 'Raw JSON' },
])
</script>
