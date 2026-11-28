<template>
	<article
		class="flex min-w-0 flex-col rounded-2xl border border-solid border-surface-4 bg-surface-3"
		:class="{ 'cursor-pointer transition-[filter] hover:brightness-125': !editDisabled }"
		@click="openEditor"
	>
		<header class="flex min-w-0 items-center gap-3 p-4 pb-1 pt-2.5">
			<h2 class="m-0 mr-auto min-w-0 break-words text-lg font-bold text-contrast">
				<NuxtLink
					v-if="!editDisabled"
					:to="editorLocation"
					class="rounded-md no-underline underline-offset-4 [color:inherit] hover:underline focus-visible:outline-brand"
				>
					{{ rule.name }}
				</NuxtLink>
				<span v-else>{{ rule.name }}</span>
			</h2>
			<div class="shrink-0">
				<TeleportOverflowMenu
					type="quiet"
					:disabled="deleteDisabled"
					:label="`Options for ${rule.name}`"
					tooltip="Rule options"
					:options="ruleOptions"
				>
					<MoreHorizontalIcon aria-hidden="true" />
				</TeleportOverflowMenu>
			</div>
		</header>

		<div class="flex min-w-0 flex-col gap-1 px-4 pb-4 pt-1">
			<div
				:id="detailsId"
				class="grid items-start gap-4 md:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]"
			>
				<pre
					class="ace-modrinth m-0 min-w-0 overflow-auto rounded-xl border border-solid border-surface-4 !bg-surface-1 p-4 text-sm leading-relaxed transition-[max-height] duration-300 motion-reduce:transition-none"
					:class="previewHeightClass"
					tabindex="0"
					aria-label="Rhai expression preview"
				><code v-if="preview"><template v-for="(line, index) in preview" :key="index"><span v-for="(token, tokenIndex) in line" :key="tokenIndex" :class="token.type.split('.').map((part) => 'ace_' + part)">{{ token.value }}</span>{{ index < preview.length - 1 ? '\n' : '' }}</template></code><code v-else>{{ rule.params.expression }}</code></pre>

				<div
					class="flex min-w-0 flex-col gap-4 overflow-y-auto transition-[max-height] duration-300 motion-reduce:transition-none"
					:class="previewHeightClass"
					role="group"
					aria-label="File scan patterns and questions"
					tabindex="0"
				>
					<section class="flex min-w-0 flex-col gap-2" aria-label="File scan patterns">
						<h3 class="m-0 text-sm font-semibold text-contrast">File scan patterns</h3>
						<dl v-if="patterns.length" class="m-0 divide-y divide-surface-4">
							<div v-for="[name, regex] in patterns" :key="name" class="flex flex-col gap-1 py-2">
								<dt class="break-words text-sm font-medium text-primary">{{ name }}</dt>
								<dd class="m-0 min-w-0 break-all text-sm text-secondary">
									<code class="whitespace-pre-wrap">{{ regex }}</code>
								</dd>
							</div>
						</dl>
						<p v-else class="m-0 text-sm text-secondary">No file scan patterns.</p>
					</section>
					<section class="flex min-w-0 flex-col gap-2" aria-label="Questions">
						<h3 class="m-0 text-sm font-semibold text-contrast">Questions</h3>
						<dl v-if="questions.length" class="m-0 divide-y divide-surface-4">
							<div
								v-for="[key, { question }] in questions"
								:key="key"
								class="flex flex-col gap-1 py-2"
							>
								<dt class="break-words text-sm font-medium text-primary">{{ key }}</dt>
								<dd class="m-0 min-w-0 whitespace-pre-wrap break-words text-sm text-secondary">
									{{ question }}
								</dd>
							</div>
						</dl>
						<p v-else class="m-0 text-sm text-secondary">No questions.</p>
					</section>
				</div>
			</div>

			<p v-if="rule.deployment_status !== 'not_deployed'" class="m-0 text-sm text-secondary">
				{{ deploymentStatus[rule.deployment_status] }}
			</p>
			<div class="mt-2 flex flex-wrap items-center gap-2">
				<RuleMetadata :rule="rule" />
				<Button
					type="quiet"
					size="sm"
					class="ml-auto shrink-0"
					:aria-expanded="expanded"
					:aria-controls="detailsId"
					@click="expanded = !expanded"
				>
					{{ expanded ? 'Show less' : 'Show more' }}
				</Button>
			</div>
		</div>
		<slot name="delete-confirmation" />
	</article>
</template>

<script setup lang="ts">
import type { ProjectFlagRule } from '@modrinth/api-client'
import { EditIcon, MoreHorizontalIcon, TrashIcon } from '@modrinth/assets'
import '@modrinth/assets/styles/ace.css'
import { Button, type ButtonMenuOption, TeleportOverflowMenu } from '@modrinth/ui'
import { computed, onMounted, ref, shallowRef, useId } from 'vue'

import type { tokenizeRhai } from './ace-rhai'
import RuleMetadata from './rule-metadata.vue'

const props = defineProps<{
	rule: ProjectFlagRule
	editDisabled?: boolean
	deleteDisabled?: boolean
}>()
const emit = defineEmits<{ requestDelete: [] }>()
const expanded = ref(false)
const previewHeightClass = computed(() => (expanded.value ? 'max-h-[600px]' : 'max-h-[180px]'))
const detailsId = useId()
const route = useRoute()
const editorLocation = computed(() => ({
	query: { ...route.query, rule: props.rule.id },
	hash: route.hash,
}))
const ruleOptions = computed<ButtonMenuOption[]>(() => [
	{
		type: 'link',
		label: 'Edit rule',
		icon: EditIcon,
		shown: !props.editDisabled,
		to: editorLocation.value,
	},
	{ label: 'Delete rule', icon: TrashIcon, tone: 'red', action: () => emit('requestDelete') },
])
const deploymentStatus = {
	current: 'Deployed · current definition',
	outdated: 'Outdated · definition differs from live',
}
const patterns = computed(() => Object.entries(props.rule.params.scan_files_for))
const questions = computed(() => Object.entries(props.rule.params.decision_questions ?? {}))
const tokenize = shallowRef<typeof tokenizeRhai>()
const preview = computed(() => tokenize.value?.(props.rule.params.expression))

function openEditor(event: MouseEvent) {
	const target = event.target as Element | null
	const card = event.currentTarget as HTMLElement
	if (
		props.editDisabled ||
		event.defaultPrevented ||
		target?.closest?.('a, button, [role="button"]') ||
		card.ownerDocument.getSelection()?.isCollapsed === false
	) {
		return
	}
	return navigateTo(editorLocation.value)
}

onMounted(async () => {
	try {
		const { tokenizeRhai } = await import('./ace-rhai')
		tokenize.value = tokenizeRhai
	} catch {
		// The plain-text preview is available if Ace fails to load.
	}
})
</script>
