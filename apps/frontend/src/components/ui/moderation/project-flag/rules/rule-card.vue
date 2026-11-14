<template>
	<article
		ref="card"
		class="flex min-w-0 cursor-pointer flex-col gap-1 rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4 outline-none transition-[filter] hover:brightness-125"
		tabindex="-1"
		@click="openEditor"
	>
		<header class="flex min-w-0 items-start justify-between gap-3">
			<h2 class="m-0 min-w-0 break-words text-lg font-bold text-contrast">
				<button
					type="button"
					class="cursor-pointer rounded-md border-0 bg-transparent p-0 text-left [color:inherit] [font:inherit] focus-visible:outline-brand"
					:aria-label="`Edit rule ${rule.name}`"
					@click.stop="openEditor"
				>
					{{ rule.name }}
				</button>
			</h2>
			<div class="shrink-0" @click.stop>
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

		<div class="grid items-start gap-4 md:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
			<pre
				class="ace-modrinth m-0 max-h-96 min-w-0 overflow-auto rounded-xl border border-solid border-surface-4 !bg-surface-1 p-4 text-sm leading-relaxed"
				tabindex="0"
				aria-label="Rhai expression preview"
			><code v-if="preview"><template v-for="(line, index) in preview" :key="index"><span v-for="(token, tokenIndex) in line" :key="tokenIndex" :class="token.type.split('.').map((part) => 'ace_' + part)">{{ token.value }}</span>{{ index < preview.length - 1 ? '\n' : '' }}</template></code><code v-else>{{ rule.params.expression }}</code></pre>

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
		</div>

		<div class="mt-2 flex flex-wrap items-center gap-1 text-sm text-secondary">
			<time :datetime="rule.updated_at" :title="formatTimestamp(rule.updated_at)">
				Last edited {{ formatDate(rule.updated_at) }}
			</time>
			<BulletDivider aria-hidden="true" />
			<time :datetime="rule.created_at" :title="formatTimestamp(rule.created_at)">
				Created {{ formatDate(rule.created_at) }}
			</time>
		</div>
		<slot name="delete-confirmation" />
	</article>
</template>

<script setup lang="ts">
import type { ProjectFlagRule } from '@modrinth/api-client'
import { MoreHorizontalIcon, TrashIcon } from '@modrinth/assets'
import '@modrinth/assets/styles/ace.css'
import { BulletDivider, type ButtonMenuOption, TeleportOverflowMenu } from '@modrinth/ui'
import { computed, onMounted, shallowRef, useTemplateRef } from 'vue'

import type { tokenizeRhai } from './ace-rhai'

const props = defineProps<{ rule: ProjectFlagRule; deleteDisabled?: boolean }>()
const emit = defineEmits<{ edit: [event: MouseEvent]; requestDelete: [] }>()
const card = useTemplateRef<HTMLElement>('card')
const ruleOptions: ButtonMenuOption[] = [
	{ label: 'Delete rule', icon: TrashIcon, tone: 'red', action: () => emit('requestDelete') },
]
const patterns = computed(() => Object.entries(props.rule.params.scan_files_for))
const tokenize = shallowRef<typeof tokenizeRhai>()
const preview = computed(() => tokenize.value?.(props.rule.params.expression))

function formatDate(value: string) {
	return new Date(value).toLocaleDateString('en-US', { dateStyle: 'medium', timeZone: 'UTC' })
}

function formatTimestamp(value: string) {
	return new Date(value).toLocaleString('en-US', {
		dateStyle: 'full',
		timeStyle: 'long',
		timeZone: 'UTC',
	})
}

onMounted(async () => {
	try {
		const { tokenizeRhai } = await import('./ace-rhai')
		tokenize.value = tokenizeRhai
	} catch {
		// The plain-text preview is available if Ace fails to load.
	}
})

function openEditor(event: MouseEvent) {
	card.value?.focus({ preventScroll: true })
	emit('edit', event)
}
</script>
