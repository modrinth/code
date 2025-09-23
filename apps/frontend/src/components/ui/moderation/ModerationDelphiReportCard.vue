<template>
	<div class="universal-card">
		<div class="flex flex-col gap-4">
			<div class="flex flex-col gap-3 sm:flex-row sm:items-center">
				<div class="flex min-w-0 flex-1 items-center gap-3">
					<Avatar :src="report.project.icon_url" size="3rem" class="flex-shrink-0" />
					<div class="min-w-0 flex-1">
						<h3 class="truncate text-lg font-semibold">{{ report.project.title }}</h3>

						<div class="flex flex-col gap-2 text-sm text-secondary sm:flex-row sm:items-center">
							<nuxt-link
								v-if="report.target"
								:to="`/${report.target.type}/${report.target.slug}`"
								class="inline-flex flex-row items-center gap-1 transition-colors duration-100 ease-in-out hover:text-brand"
							>
								<Avatar
									:src="report.target.avatar_url"
									:circle="report.target.type === 'user'"
									size="1rem"
									class="flex-shrink-0"
								/>
								<span class="truncate">
									<OrganizationIcon
										v-if="report.target.type === 'organization'"
										class="align-middle"
									/>
									{{ report.target.name }}
								</span>
							</nuxt-link>

							<div class="flex flex-wrap items-center gap-2">
								<span
									class="whitespace-nowrap rounded-full bg-button-bg p-0.5 px-2 text-xs font-semibold text-secondary"
								>
									Score: {{ report.priority_score }}
								</span>
								<span
									class="whitespace-nowrap rounded-full bg-button-bg p-0.5 px-2 text-xs font-semibold"
									:class="{
										'text-brand': report.status === 'approved',
										'text-red': report.status === 'rejected',
										'text-secondary': report.status === 'pending',
									}"
								>
									{{ report.status.charAt(0).toUpperCase() + report.status.slice(1) }}
								</span>
								<span class="max-w-[200px] truncate font-mono text-xs sm:max-w-none">
									{{
										report.version.files.find((file) => file.primary)?.filename ||
										'Unknown primary file'
									}}
								</span>
							</div>
						</div>
					</div>
				</div>

				<div
					class="mt-2 flex flex-col items-stretch gap-2 sm:mt-0 sm:flex-row sm:items-center sm:gap-2"
				>
					<span class="hidden whitespace-nowrap text-sm text-secondary sm:block">
						{{ formatRelativeTime(dayjs(report.detected_at).toDate()) }}
					</span>

					<div class="flex flex-col gap-2 sm:flex-row">
						<div class="flex gap-2">
							<ButtonStyled class="flex-1 sm:flex-none">
								<button
									v-tooltip="!isPending ? 'This report has already been dealt with.' : undefined"
									:disabled="!isPending"
									class="w-full sm:w-auto"
								>
									Accept
								</button>
							</ButtonStyled>
							<ButtonStyled class="flex-1 sm:flex-none">
								<button
									v-tooltip="!isPending ? 'This report has already been dealt with.' : undefined"
									:disabled="!isPending"
									class="w-full sm:w-auto"
								>
									Reject
								</button>
							</ButtonStyled>
						</div>

						<div class="flex justify-center gap-2 sm:justify-start">
							<ButtonStyled circular>
								<nuxt-link :to="versionUrl">
									<EyeIcon />
								</nuxt-link>
							</ButtonStyled>
							<ButtonStyled circular>
								<OverflowMenu :options="quickActions">
									<template #default>
										<EllipsisVerticalIcon />
									</template>
									<template #copy-id>
										<ClipboardCopyIcon />
										<span class="hidden sm:inline">Copy ID</span>
									</template>
									<template #copy-link>
										<LinkIcon />
										<span class="hidden sm:inline">Copy link</span>
									</template>
								</OverflowMenu>
							</ButtonStyled>
						</div>
					</div>
				</div>
			</div>

			<div class="text-sm text-secondary sm:hidden">
				{{ formatRelativeTime(dayjs(report.detected_at).toDate()) }}
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	ClipboardCopyIcon,
	EllipsisVerticalIcon,
	EyeIcon,
	LinkIcon,
	OrganizationIcon,
} from '@modrinth/assets'
import type { ExtendedDelphiReport } from '@modrinth/moderation'
import {
	Avatar,
	ButtonStyled,
	injectNotificationManager,
	OverflowMenu,
	type OverflowMenuOption,
	useRelativeTime,
} from '@modrinth/ui'
import dayjs from 'dayjs'

const { addNotification } = injectNotificationManager()

const props = defineProps<{
	report: ExtendedDelphiReport
}>()

const formatRelativeTime = useRelativeTime()
const isPending = computed(() => props.report.status === 'pending')

const quickActions: OverflowMenuOption[] = [
	{
		id: 'copy-link',
		action: () => {
			const base = window.location.origin
			const reviewUrl = `${base}/moderation/tech-reviews?q=${props.report.version.id}`
			navigator.clipboard.writeText(reviewUrl).then(() => {
				addNotification({
					type: 'success',
					title: 'Tech review link copied',
					text: 'The link to this tech review has been copied to your clipboard.',
				})
			})
		},
	},
	{
		id: 'copy-id',
		action: () => {
			navigator.clipboard.writeText(props.report.version.id).then(() => {
				addNotification({
					type: 'success',
					title: 'Version ID copied',
					text: 'The ID of this version has been copied to your clipboard.',
				})
			})
		},
	},
]

const versionUrl = computed(() => {
	return `/${props.report.project.project_type}/${props.report.project.slug}/version/${props.report.version.id}`
})
</script>

<style lang="scss" scoped></style>
