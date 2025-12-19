<template>
	<section class="experimental-styles-within overflow-visible">
		<Admonition v-if="!hideVersionsAdmonition && currentMember" type="info" class="mb-4">
			Managing project versions has moved! You can now add and edit versions in the
			<NuxtLink to="settings/versions" class="font-medium text-blue hover:underline"
				>project settings</NuxtLink
			>.
			<template #actions>
				<div class="flex gap-2">
					<ButtonStyled color="blue">
						<button
							aria-label="Project Settings"
							class="!shadow-none"
							@click="() => router.push('settings/versions')"
						>
							<SettingsIcon />
							Edit versions
						</button>
					</ButtonStyled>
					<ButtonStyled type="transparent">
						<button
							aria-label="Dismiss"
							class="!shadow-none"
							@click="() => (hideVersionsAdmonition = true)"
						>
							Dismiss
						</button>
					</ButtonStyled>
				</div>
			</template>
		</Admonition>

		<ProjectPageVersions
			v-if="versions.length"
			:project="project"
			:versions="versions"
			:show-files="flags.showVersionFilesInTable"
			:current-member="!!currentMember"
			:loaders="tags.loaders"
			:game-versions="tags.gameVersions"
			:base-id="baseDropdownId"
			:version-link="
				(version) =>
					`/${project.project_type}/${
						project.slug ? project.slug : project.id
					}/version/${encodeURI(version.displayUrlEnding)}`
			"
		>
			<template #actions="{ version }">
				<ButtonStyled circular type="transparent">
					<a
						v-tooltip="`Download`"
						:href="getPrimaryFile(version).url"
						class="hover:!bg-button-bg [&>svg]:!text-green"
						aria-label="Download"
						@click="emit('onDownload')"
					>
						<DownloadIcon aria-hidden="true" />
					</a>
				</ButtonStyled>
				<ButtonStyled circular type="transparent">
					<OverflowMenu
						class="hover:!bg-button-bg"
						:dropdown-id="`${baseDropdownId}-${version.id}`"
						:options="[
							{
								id: 'download',
								color: 'primary',
								hoverFilled: true,
								link: getPrimaryFile(version).url,
								action: () => {
									emit('onDownload')
								},
							},
							{
								id: 'new-tab',
								action: () => {},
								link: `/${project.project_type}/${
									project.slug ? project.slug : project.id
								}/version/${encodeURI(version.displayUrlEnding)}`,
								external: true,
							},
							{
								id: 'copy-link',
								action: () =>
									copyToClipboard(
										`https://modrinth.com/${project.project_type}/${
											project.slug ? project.slug : project.id
										}/version/${encodeURI(version.displayUrlEnding)}`,
									),
							},
							{
								id: 'share',
								action: () => {},
								shown: false,
							},
							{
								id: 'report',
								color: 'red',
								hoverFilled: true,
								action: () => (auth.user ? reportVersion(version.id) : navigateTo('/auth/sign-in')),
								shown: !currentMember,
							},
							{ divider: true, shown: currentMember || flags.developerMode },
							{
								id: 'copy-id',
								action: () => {
									copyToClipboard(version.id)
								},
								shown: currentMember || flags.developerMode,
							},
							{
								id: 'copy-maven',
								action: () => {
									copyToClipboard(`maven.modrinth:${project.slug}:${version.id}`)
								},
								shown: flags.developerMode,
							},
						]"
						aria-label="More options"
					>
						<MoreVerticalIcon aria-hidden="true" />
						<template #download>
							<DownloadIcon aria-hidden="true" />
							Download
						</template>
						<template #new-tab>
							<ExternalIcon aria-hidden="true" />
							Open in new tab
						</template>
						<template #copy-link>
							<LinkIcon aria-hidden="true" />
							Copy link
						</template>
						<template #share>
							<ShareIcon aria-hidden="true" />
							Share
						</template>
						<template #report>
							<ReportIcon aria-hidden="true" />
							Report
						</template>
						<template #copy-id>
							<ClipboardCopyIcon aria-hidden="true" />
							Copy ID
						</template>
						<template #copy-maven>
							<ClipboardCopyIcon aria-hidden="true" />
							Copy Maven coordinates
						</template>
					</OverflowMenu>
				</ButtonStyled>
			</template>
		</ProjectPageVersions>
		<template v-else>
			<p class="ml-2">
				No versions in project. Visit
				<NuxtLink to="settings/versions">
					<span class="font-medium text-green hover:underline">project settings</span> to
				</NuxtLink>
				upload your first version.
			</p>
		</template>
	</section>
</template>

<script setup>
import {
	ClipboardCopyIcon,
	DownloadIcon,
	ExternalIcon,
	LinkIcon,
	MoreVerticalIcon,
	ReportIcon,
	SettingsIcon,
	ShareIcon,
} from '@modrinth/assets'
import { Admonition, ButtonStyled, OverflowMenu, ProjectPageVersions } from '@modrinth/ui'
import { useLocalStorage } from '@vueuse/core'

import { reportVersion } from '~/utils/report-helpers.ts'

const props = defineProps({
	project: {
		type: Object,
		default() {
			return {}
		},
	},
	versions: {
		type: Array,
		default() {
			return []
		},
	},
	currentMember: {
		type: Object,
		default() {
			return null
		},
	},
})

const tags = useGeneratedState()
const flags = useFeatureFlags()
const auth = await useAuth()

const hideVersionsAdmonition = useLocalStorage(
	'hideVersionsHasMovedAdmonition',
	!props.versions.length,
)

const emit = defineEmits(['onDownload', 'deleteVersion'])

const router = useNativeRouter()

const baseDropdownId = useId()

function getPrimaryFile(version) {
	return version.files.find((x) => x.primary) || version.files[0]
}

async function copyToClipboard(text) {
	await navigator.clipboard.writeText(text)
}
</script>
