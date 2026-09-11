<template>
	<div class="flex min-h-0 flex-col">
		<!-- History -->
		<div ref="historyEl" class="min-h-0 flex-1 overflow-y-auto px-1">
			<ConversationThread
				v-if="thread"
				:thread="thread"
				:project="projectV2"
				:current-member="currentMember ?? undefined"
				:auth="auth"
				hide-actions
				@update-thread="() => invalidate()"
			/>
			<div v-else class="flex items-center gap-2 py-6 text-secondary">
				<SpinnerIcon class="size-4 animate-spin" /> Loading thread…
			</div>
		</div>

		<!-- Merged composer -->
		<div class="shrink-0 border-0 border-t border-solid border-divider bg-surface-2">
			<button
				class="flex w-full items-center gap-2 px-2 py-1.5 text-xs font-semibold text-secondary hover:text-contrast"
				@click="composerOpen = !composerOpen"
			>
				<ChevronUpIcon v-if="!composerOpen" class="size-4" />
				<ChevronDownIcon v-else class="size-4" />
				{{ composerOpen ? 'Hide composer' : composerSummary }}
			</button>

			<div v-show="composerOpen" class="flex flex-col gap-2 px-2 pb-2">
				<div class="flex items-center gap-2">
					<div class="flex gap-1 rounded-md bg-bg p-0.5">
						<button
							v-for="m in MODES"
							:key="m.id"
							class="rounded px-2 py-0.5 text-xs font-semibold"
							:class="
								mode === m.id ? 'bg-surface-2 text-contrast' : 'text-secondary hover:text-contrast'
							"
							@click="mode = m.id"
						>
							{{ m.label }}
						</button>
					</div>
					<button
						v-tooltip="engine.useSimpleEditor.value ? 'Rich editor' : 'Plain text'"
						class="rounded p-1 text-secondary hover:bg-button-bg hover:text-contrast"
						aria-label="Toggle editor mode"
						@click="engine.useSimpleEditor.value = !engine.useSimpleEditor.value"
					>
						<ToggleLeftIcon v-if="engine.useSimpleEditor.value" class="size-4" />
						<ToggleRightIcon v-else class="size-4" />
					</button>
					<Button
						v-if="mode === 'decision'"
						size="xs"
						class="ml-auto"
						:disabled="engine.loadingMessage.value"
						@click="engine.generateMessage()"
					>
						<SpinnerIcon v-if="engine.loadingMessage.value" class="animate-spin" />
						<RedoIcon v-else />
						{{ engine.generatedMessage.value ? 'Regenerate' : 'Generate from checklist' }}
					</Button>
				</div>

				<MarkdownEditor
					v-if="!engine.useSimpleEditor.value"
					v-model="text"
					:max-height="160"
					:placeholder="composerPlaceholder"
					:disabled="false"
					:heading-buttons="false"
					:on-image-upload="engine.onUploadHandler"
				/>
				<Textarea
					v-else
					v-model="text"
					:placeholder="composerPlaceholder"
					autocomplete="off"
					input-class="h-28 font-mono text-sm"
				/>

				<div class="flex items-center gap-1.5">
					<template v-if="engine.done.value">
						<Button
							type="colored"
							color="brand"
							size="sm"
							class="flex-1"
							@click="engine.endChecklist()"
						>
							<RightArrowIcon v-if="engine.hasNextProject.value" />
							<CheckIcon v-else />
							{{ engine.hasNextProject.value ? 'Next project' : 'All done' }}
						</Button>
					</template>

					<template v-else-if="mode === 'reply'">
						<Button
							type="colored"
							color="brand"
							size="sm"
							class="flex-1"
							:disabled="!text.trim() || sending"
							@click="sendReply"
						>
							<SpinnerIcon v-if="sending" class="animate-spin" />
							<component :is="replyAs === 'private' ? EyeOffIcon : ReplyIcon" v-else />
							{{ replyAs === 'private' ? 'Add private note' : 'Send reply' }}
						</Button>
						<div class="flex overflow-hidden rounded-md border border-solid border-divider text-xs">
							<button
								v-for="opt in REPLY_AS"
								:key="opt.id"
								class="px-2 py-1 font-semibold"
								:class="
									replyAs === opt.id
										? 'bg-button-bg text-contrast'
										: 'text-secondary hover:text-contrast'
								"
								@click="replyAs = opt.id"
							>
								{{ opt.label }}
							</button>
						</div>
					</template>

					<template v-else>
						<Button
							type="colored"
							color="red"
							size="sm"
							class="flex-1"
							:disabled="engine.loadingModerationDecision.value"
							@click="engine.sendMessage('rejected')"
						>
							<SpinnerIcon
								v-if="engine.moderationDecision.value === 'rejected'"
								class="animate-spin"
							/>
							<XIcon v-else />
							Reject
						</Button>
						<Button
							type="colored"
							color="orange"
							size="sm"
							class="flex-1"
							:disabled="engine.loadingModerationDecision.value"
							@click="engine.sendMessage('withheld')"
						>
							<SpinnerIcon
								v-if="engine.moderationDecision.value === 'withheld'"
								class="animate-spin"
							/>
							<EyeOffIcon v-else />
							Withhold
						</Button>
						<Button
							type="colored"
							color="green"
							size="sm"
							class="flex-1"
							:disabled="engine.loadingModerationDecision.value"
							@click="engine.sendMessage(engine.approveSendStatus.value)"
						>
							<SpinnerIcon
								v-if="engine.moderationDecision.value === engine.approveSendStatus.value"
								class="animate-spin"
							/>
							<CheckIcon v-else />
							Approve
						</Button>
					</template>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	CheckIcon,
	ChevronDownIcon,
	ChevronUpIcon,
	EyeOffIcon,
	RedoIcon,
	ReplyIcon,
	RightArrowIcon,
	SpinnerIcon,
	ToggleLeftIcon,
	ToggleRightIcon,
	XIcon,
} from '@modrinth/assets'
import { Button, injectProjectPageContext, MarkdownEditor, Textarea } from '@modrinth/ui'
import { computed, nextTick, ref, watch } from 'vue'

import { injectModerationChecklist } from '~/components/ui/moderation/checklist/checklist-context'
import ConversationThread from '~/components/ui/thread/ConversationThread.vue'

const engine = injectModerationChecklist()
const { projectV2, thread, currentMember, invalidate } = injectProjectPageContext()
const auth = await useAuth()

const MODES = [
	{ id: 'reply' as const, label: 'Reply' },
	{ id: 'decision' as const, label: 'Decision' },
]
const REPLY_AS = [
	{ id: 'public' as const, label: 'Public' },
	{ id: 'private' as const, label: 'Private' },
]

const mode = ref<'reply' | 'decision'>('reply')
const replyAs = ref<'public' | 'private'>('public')
const sending = ref(false)
const composerOpen = ref(true)
const historyEl = ref<HTMLElement | null>(null)

const text = computed({
	get: () => engine.message.value ?? '',
	set: (v: string) => {
		engine.message.value = v === '' ? null : v
	},
})

watch(
	() => engine.generatedMessage.value,
	(has) => {
		if (has) {
			mode.value = 'decision'
			composerOpen.value = true
		}
	},
)

// Keep the thread scrolled to the newest message as it loads / updates.
watch(
	() => thread.value?.messages?.length,
	async () => {
		await nextTick()
		if (historyEl.value) historyEl.value.scrollTop = historyEl.value.scrollHeight
	},
	{ immediate: true },
)

const composerPlaceholder = computed(() =>
	mode.value === 'reply'
		? replyAs.value === 'private'
			? 'Private note for other moderators…'
			: 'Message the project owner…'
		: 'Generate the decision message from the checklist, or write one.',
)

const composerSummary = computed(() => {
	const t = text.value.trim()
	if (t) return `Draft: ${t.slice(0, 40)}${t.length > 40 ? '…' : ''}`
	return mode.value === 'decision' ? 'Compose decision' : 'Reply to thread'
})

async function sendReply() {
	if (!text.value.trim() || sending.value) return
	sending.value = true
	const ok = await engine.postThreadReply(text.value, replyAs.value === 'private')
	sending.value = false
	if (ok) engine.message.value = null
}
</script>
