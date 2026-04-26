<script setup lang="ts">
import { CopyIcon, LibraryIcon, PlayIcon, SearchIcon } from '@modrinth/assets'
import { ButtonStyled, Card, NewModal, StyledInput } from '@modrinth/ui'
import { computed, onMounted, ref } from 'vue'

import emails from '~/templates/emails'

const allTemplates = Object.keys(emails).sort()
const query = ref('')
const filtered = computed(() =>
	allTemplates.filter((t) => t.toLowerCase().includes(query.value.toLowerCase().trim())),
)

function openAll() {
	let offset = 0
	for (const id of filtered.value) {
		openPopupPreview(id, offset)
		offset++
	}
}

function copy(id: string) {
	navigator.clipboard?.writeText(`/_internal/templates/email/${id}`).catch(() => {})
}

const previewModal = ref<{ hide: () => void; show: () => void } | null>(null)
const previewTemplate = ref<string | null>(null)
const previewLoading = ref(false)
const previewError = ref<string | null>(null)
const previewHtml = ref('')
const previewVariables = ref<string[]>([])
const variableValues = ref<Record<string, string>>({})

function extractVariables(html: string): string[] {
	const tokens = new Set<string>()
	const regex = /\{([a-zA-Z0-9_.-]+)\}/g
	let match = regex.exec(html)

	while (match !== null) {
		tokens.add(match[1])
		match = regex.exec(html)
	}

	return [...tokens]
}

const renderedPreview = computed(() => {
	let html = previewHtml.value

	for (const [key, value] of Object.entries(variableValues.value)) {
		if (!value) {
			continue
		}

		const pattern = new RegExp(`\\{${key.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\}`, 'g')
		html = html.replace(pattern, value)
	}

	return html
})

async function openPreview(id: string, event?: MouseEvent) {
	if (event?.shiftKey) {
		openPopupPreview(id)
		return
	}

	previewTemplate.value = id
	previewLoading.value = true
	previewError.value = null
	previewHtml.value = ''
	previewVariables.value = []
	variableValues.value = {}

	try {
		const response = await fetch(`/_internal/templates/email/${id}`)
		previewHtml.value = await response.text()

		if (!response.ok) {
			throw new Error(`Failed to load template ${id}`)
		}

		const variables = extractVariables(previewHtml.value)
		previewVariables.value = variables
		variableValues.value = Object.fromEntries(variables.map((value) => [value, '']))
		previewModal.value?.show()
	} catch (error) {
		previewError.value = 'Failed to load email preview.'
		console.error(error)
		previewModal.value?.show()
	} finally {
		previewLoading.value = false
	}
}

function closePreview() {
	previewModal.value?.hide()
}

function openPopupPreview(id: string, offset = 0) {
	const width = 600
	const height = 850
	const left = window.screenX + (window.outerWidth - width) / 2 + ((offset * 28) % 320)
	const top = window.screenY + (window.outerHeight - height) / 2 + ((offset * 28) % 320)
	window.open(
		`/_internal/templates/email/${id}`,
		`email-${id}`,
		`popup=yes,width=${width},height=${height},left=${left},top=${top},resizable=yes,scrollbars=yes,menubar=no,toolbar=no,location=no,status=no`,
	)
}

const counts = computed(() => ({
	total: allTemplates.length,
	shown: filtered.value.length,
}))

onMounted(() => {
	document.getElementById('email-search')?.focus()
})
</script>

<template>
	<div class="normal-page no-sidebar">
		<h1 class="mb-4 text-3xl font-extrabold text-heading">Email templates</h1>
		<NewModal
			ref="previewModal"
			header="Preview email"
			width="min(92vw, 1000px)"
			:max-content-height="'88vh'"
			scrollable
		>
			<div class="flex flex-col gap-4">
				<p class="label__title text-base">Template: {{ previewTemplate }}</p>

				<div
					v-if="previewError"
					class="border-danger bg-danger/10 text-danger my-2 rounded border px-3 py-2 text-sm"
				>
					{{ previewError }}
				</div>

				<div v-if="previewLoading" class="my-4 text-sm text-secondary">Loading preview…</div>
				<div v-else>
					<div v-if="previewVariables.length" class="mt-2 grid gap-3 md:grid-cols-2">
						<label
							v-for="variable in previewVariables"
							:key="variable"
							:for="`preview-${variable}`"
							class="flex flex-col"
						>
							<span class="label__title">{{ variable }}</span>
							<StyledInput
								:id="`preview-${variable}`"
								v-model="variableValues[variable]"
								type="text"
								:placeholder="`Enter ${variable}`"
							/>
						</label>
					</div>
					<p v-else class="mt-2 text-xs text-secondary">
						No template variables were detected; preview shown using default values.
					</p>

					<div class="mt-4">
						<div class="label__title mb-2">Rendered template</div>
						<iframe
							v-if="!previewError"
							:srcdoc="renderedPreview"
							class="h-[60vh] w-full rounded border border-divider bg-white"
							sandbox="allow-same-origin"
						/>
						<div
							v-else
							class="rounded border border-divider bg-white px-4 py-3 text-sm text-secondary"
						>
							Could not render template preview.
						</div>
					</div>

					<div class="input-group mt-4">
						<button class="iconified-button transparent" type="button" @click="closePreview">
							Close
						</button>
					</div>
				</div>
			</div>
		</NewModal>
		<div class="normal-page__content">
			<Card class="mb-6 flex flex-col gap-4">
				<div class="flex flex-wrap items-center gap-3">
					<StyledInput
						id="email-search"
						v-model="query"
						type="search"
						:icon="SearchIcon"
						placeholder="Search templates..."
						wrapper-class="w-72"
					/>

					<ButtonStyled color="brand">
						<button :disabled="filtered.length === 0" @click="openAll">
							<LibraryIcon class="h-4 w-4" aria-hidden="true" />
							Open all ({{ counts.shown }})
						</button>
					</ButtonStyled>

					<span class="text-sm text-secondary">
						Showing <span class="font-medium text-contrast">{{ counts.shown }}</span> of
						<span class="font-medium text-contrast">{{ counts.total }}</span>
					</span>
				</div>

				<div
					v-if="filtered.length === 0"
					class="rounded-lg border border-dashed border-divider px-6 py-10 text-center text-sm text-secondary"
				>
					No templates match your search.
				</div>

				<ul v-else class="grid gap-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
					<li
						v-for="id in filtered"
						:key="id"
						class="hover:border-green/70 group flex flex-col justify-between rounded-lg border border-divider bg-button-bg p-4 shadow-sm transition hover:shadow"
					>
						<div class="mb-3">
							<div class="font-mono text-sm font-semibold tracking-tight text-contrast">
								{{ id }}
							</div>
							<div class="mt-1 truncate text-xs text-secondary">
								/_internal/templates/email/{{ id }}
							</div>
						</div>

						<div class="mt-auto flex gap-2">
							<ButtonStyled color="brand" class="flex-1">
								<button class="w-full justify-center" @click="openPreview(id, $event)">
									<PlayIcon class="h-4 w-4" aria-hidden="true" />
									Preview
								</button>
							</ButtonStyled>

							<ButtonStyled>
								<button class="justify-center" title="Copy preview URL" @click="copy(id)">
									<CopyIcon class="h-4 w-4" aria-hidden="true" />
								</button>
							</ButtonStyled>
						</div>
					</li>
				</ul>
			</Card>

			<p class="mt-2 text-xs text-secondary">
				All templates come from
				<code class="rounded bg-code-bg px-1 py-0.5 text-[11px] text-code-text"
					>src/emails/index.ts</code
				>. Popouts render via
				<code class="rounded bg-code-bg px-1 py-0.5 text-[11px] text-code-text"
					>/_internal/templates/email/[template]</code
				>.
			</p>
		</div>
	</div>
</template>
