<script setup lang="ts">
import { CopyIcon, LibraryIcon, PlayIcon, SearchIcon } from '@modrinth/assets'
import { ButtonStyled, Card } from '@modrinth/ui'
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
		openPreview(id, offset)
		offset++
	}
}

function copy(id: string) {
	navigator.clipboard?.writeText(`/_internal/templates/email/${id}`).catch(() => {})
}

function openPreview(id: string, offset = 0) {
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
		<div class="normal-page__content">
			<Card class="mb-6 flex flex-col gap-4">
				<div class="flex flex-wrap items-center gap-3">
					<div class="relative">
						<SearchIcon
							class="pointer-events-none absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 text-secondary"
						/>
						<input
							id="email-search"
							v-model="query"
							type="text"
							placeholder="Search templates..."
							class="w-72 rounded-lg border border-divider bg-bg px-7 py-2 text-sm text-primary placeholder-secondary focus:border-green focus:outline-none"
						/>
					</div>

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
								<button class="w-full justify-center" @click="openPreview(id)">
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
