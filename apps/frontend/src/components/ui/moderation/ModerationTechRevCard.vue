<script setup lang="ts">
import { CheckIcon, MoreVerticalIcon, ScanEyeIcon, XIcon } from '@modrinth/assets'
import { Avatar, ButtonStyled } from '@modrinth/ui'
import { computed, ref } from 'vue'
import type { DelphiIssueResult } from '~/helpers/tech-review.dummy'

const props = defineProps<{
	item: DelphiIssueResult
}>()

type Tab = 'Summary' | 'Thread' | 'Files'
const tabs: readonly Tab[] = ['Summary', 'Thread', 'Files']
const currentTab = ref<Tab>('Summary')

const severityColor = computed(() => {
	switch (props.item.report.severity) {
		case 'SEVERE':
			return 'text-red bg-highlight-red border-solid border-[1px] border-red'
		case 'HIGH':
			return 'text-orange bg-highlight-orange border-solid border-[1px] border-orange'
		case 'MEDIUM':
			return 'text-blue bg-highlight-blue border-solid border-[1px] border-blue'
		case 'LOW':
		default:
			return 'text-green bg-highlight-green border-solid border-[1px] border-green'
	}
})

const statusColor = computed(() => {
	switch (props.item.issue.status) {
		case 'pending':
			return 'text-orange bg-orange/10'
		case 'approved':
			return 'text-green bg-green/10'
		case 'rejected':
			return 'text-red bg-red/10'
	}
})

const createdAt = computed(() => new Date(props.item.report.created).toLocaleDateString())
</script>

<template>
	<div
		class="overflow-hidden rounded-2xl border-[1px] border-solid border-surface-l4 bg-surface-l3"
	>
		<div class="flex flex-col gap-4 bg-surface-l3 p-4">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-4">
					<Avatar
						src="https://cdn.modrinth.com/data/AANobbMI/295862f4724dc3f78df3447ad6072b2dcd3ef0c9_96.webp"
						class="border-[1px] border-solid border-surface-l5 bg-surface-l4"
						size="4rem"
					/>

					<div class="flex flex-col gap-1">
						<div class="flex items-center gap-2">
							<span class="my-auto text-xl font-semibold text-contrast">Sodium</span>
							<div
								class="flex items-center gap-1 rounded-full border-[1px] border-solid border-surface-l5 bg-surface-l4 px-2.5 py-1"
							>
								<div class="h-4 w-4 rounded-full bg-surface-l5"></div>
								<span class="text-sm font-medium text-secondary">MPK</span>
							</div>

							<div
								class="rounded-full border-[1px] border-solid border-surface-l5 bg-surface-l4 px-2.5 py-1"
							>
								<span class="text-sm font-medium text-secondary">Auto-Flagged</span>
							</div>

							<div
								class="rounded-full border-[1px] border-solid px-2.5 py-1"
								:class="severityColor"
							>
								<span class="text-sm font-medium">{{
									item.report.severity.charAt(0) + item.report.severity.slice(1).toLowerCase()
								}}</span>
							</div>
						</div>

						<div class="flex items-center gap-1">
							<Avatar
								src="https://cdn.modrinth.com/user/TEZXhE2U/f4705a5f2388c65029ae2e59f1434b3e6e4de23a.png"
								class="border-[1px] border-solid border-surface-l5 bg-surface-l4"
								size="1.5rem"
								circle
							/>
							<span class="text-sm font-medium text-secondary">JellySquid</span>
						</div>
					</div>
				</div>

				<div class="flex items-center gap-3">
					<span class="text-base text-secondary">{{ createdAt }}</span>
					<div class="flex items-center gap-2">
						<ButtonStyled circular color="green">
							<button><CheckIcon /></button>
						</ButtonStyled>

						<ButtonStyled circular color="red">
							<button><XIcon /></button>
						</ButtonStyled>

						<ButtonStyled>
							<button>
								<ScanEyeIcon />
								<span class="font-semibold">Scan</span>
							</button>
						</ButtonStyled>

						<ButtonStyled circular>
							<button>
								<MoreVerticalIcon />
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>

			<hr class="border-surface-l5" />

			<!-- todo -->
			<div class="flex items-center gap-3">
				<button
					v-for="tab in tabs"
					:key="tab"
					class="border-b-2 px-4 py-2 text-base font-semibold transition-colors"
					:class="{
						'border-white text-white': currentTab === tab,
						'border-transparent text-secondary': currentTab !== tab,
					}"
					@click="currentTab = tab"
				>
					{{ tab }}
				</button>
			</div>
		</div>

		<div class="min-h-[200px] border-t border-surface-l3 bg-surface-l2 p-4">
			<div v-if="currentTab === 'Summary'" class="flex min-h-[200px] items-center justify-center">
				<div class="text-center text-secondary">
					<p class="mt-1 text-sm">TBD</p>
				</div>
			</div>

			<div
				v-else-if="currentTab === 'Thread'"
				class="flex min-h-[200px] items-center justify-center"
			>
				<div class="text-center text-secondary">
					<p class="mt-1 text-sm">TBD</p>
				</div>
			</div>

			<div v-else class="flex min-h-[200px] items-center justify-center">
				<div class="text-center text-secondary">
					<p class="mt-1 text-sm">TBD</p>
				</div>
			</div>
		</div>
	</div>
</template>
