<template>
	<div>
		<div class="flex flex-col gap-3">
			<div class="flex items-start justify-between gap-4">
				<div class="flex flex-col gap-1">
					<div class="text-xl font-semibold text-contrast">Server compatibility</div>
					<div v-if="!content" class="text-sm text-secondary">
						Select whether your server is vanilla or modded and which versions it supports. The
						Modrinth App uses this when a player joins.
					</div>
					<div v-else>
						<div v-if="content.kind === 'vanilla'" class="flex items-center gap-1.5">
							<div
								class="flex h-6 w-6 shrink-0 items-center justify-center rounded-md border border-solid border-surface-5 bg-surface-4 font-medium"
							>
								<BoxIcon class="h-4 w-4 shrink-0 text-secondary" />
							</div>

							Vanilla server
						</div>
						<div
							v-else-if="content.kind === 'modpack' && !usingCustomMrpack"
							class="flex items-center gap-1.5"
						>
							<div
								class="flex h-6 w-6 shrink-0 items-center justify-center rounded-md border border-solid border-surface-5 bg-surface-4 font-medium"
							>
								<PackageIcon class="h-4 w-4 shrink-0 text-secondary" />
							</div>
							Published modpack
						</div>
						<div v-else class="flex items-center gap-1.5">
							<div
								class="flex h-6 w-6 shrink-0 items-center justify-center rounded-md border border-solid border-surface-5 bg-surface-4 font-medium"
							>
								<PackagePlusIcon class="h-4 w-4 shrink-0 text-secondary" />
							</div>
							Custom modpack
						</div>
					</div>
				</div>
				<ButtonStyled v-if="content" type="outlined">
					<button class="!border-[1px]" @click="handleSwitchCompatibility">
						<ArrowLeftRightIcon />
						Switch type
					</button>
				</ButtonStyled>
				<ButtonStyled v-else>
					<button @click="handleSetCompatibility">
						<ComponentIcon />
						Set compatibility
					</button>
				</ButtonStyled>
			</div>

			<div
				v-if="content"
				class="flex justify-between rounded-2xl border border-solid border-surface-5 p-4"
			>
				<!-- kind = vanilla -->
				<div
					v-if="content?.kind === 'vanilla'"
					class="flex flex-col items-start justify-between gap-2.5"
				>
					<div class="flex flex-col gap-2">
						<div class="font-medium text-secondary">Recommended version</div>
						<div class="text-2xl font-semibold text-contrast">
							{{ content.recommended_game_version ?? '—' }}
						</div>
					</div>
					<div class="flex flex-col gap-2">
						<div class="font-medium text-secondary">Supported versions</div>
						<div class="flex flex-wrap gap-1.5">
							<TagItem v-for="v in content.supported_game_versions" :key="v">
								{{ v }}
							</TagItem>
							<div v-if="!content.supported_game_versions.length">—</div>
						</div>
					</div>
				</div>
				<!-- kind = modpack -->
				<div
					v-if="content?.kind === 'modpack' && modpackProject"
					class="flex w-full max-w-[500px] flex-col items-start justify-between gap-4"
				>
					<div class="flex w-full flex-col gap-2">
						<div class="font-medium text-secondary">Required modpack</div>
						<div class="w-fullitems-center flex gap-3 rounded-2xl bg-surface-1 p-3">
							<Avatar
								v-if="!usingCustomMrpack"
								:src="modpackProject.icon_url"
								size="56px"
								:tint-by="modpackProject.name"
							/>
							<div
								v-else
								class="flex h-14 w-14 shrink-0 items-center justify-center rounded-xl border border-solid border-surface-5 bg-surface-3"
							>
								<PackagePlusIcon class="h-10 w-10 shrink-0 text-secondary" />
							</div>

							<div class="flex flex-col">
								<div class="font-semibold text-contrast">
									{{ usingCustomMrpack ? modpackFileName : modpackProject.name }}
								</div>
								<div class="flex h-6 items-center gap-2 text-secondary">
									<Avatar
										v-if="modpackOrg?.icon_url"
										:src="modpackOrg.icon_url"
										size="24px"
										circle
									/>
									<span v-if="modpackOrg?.name">
										{{ modpackOrg.name }}
									</span>
									<div
										v-if="modpackOrg?.name && modpackVersion"
										class="h-1.5 w-1.5 rounded-full bg-surface-5"
									></div>
									<span v-if="modpackVersion">v{{ modpackVersion.version_number }}</span>
								</div>
							</div>
						</div>
					</div>
					<div v-if="modpackVersion" class="flex flex-col gap-2">
						<div class="font-medium text-secondary">Required version</div>
						<div class="flex flex-wrap gap-1.5">
							<TagItem v-for="gv in modpackVersion.game_versions" :key="gv">
								{{ gv }}
							</TagItem>
							<TagItem
								v-for="loader in modpackVersion.mrpack_loaders"
								:key="loader"
								:style="`--_color: var(--color-platform-${loader})`"
							>
								<component
									:is="getLoaderIcon(loader)"
									v-if="getLoaderIcon(loader)"
									class="h-4 w-4"
								/>
								<FormattedTag :tag="loader" enforce-type="loader" />
							</TagItem>
						</div>
					</div>
				</div>

				<ButtonStyled v-if="content">
					<button @click="handleUpdateContent" class="!w-full !max-w-[160px]">
						<RefreshCwIcon />
						Update
					</button>
				</ButtonStyled>
			</div>
		</div>
		<ServerCompatibilityModal ref="serverCompatibilityModal" />
	</div>
</template>

<script setup lang="ts">
import {
	ArrowLeftRightIcon,
	BoxIcon,
	ComponentIcon,
	getLoaderIcon,
	PackageIcon,
	PackagePlusIcon,
	RefreshCwIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	FormattedTag,
	injectModrinthClient,
	injectProjectPageContext,
	TagItem,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'

import ServerCompatibilityModal from './ServerCompatibilityModal/ServerCompatibilityModal.vue'

const serverCompatibilityModal = useTemplateRef<InstanceType<typeof ServerCompatibilityModal>>(
	'serverCompatibilityModal',
)

const { projectV3 } = injectProjectPageContext()
const { labrinth } = injectModrinthClient()

const content = computed(() => {
	if (!projectV3.value) return null

	const content = projectV3.value.minecraft_java_server?.content
	if (!content) return null

	if (content?.kind === 'vanilla' && !content.recommended_game_version) {
		return null
	}

	return content
})

const modpackVersionId = computed(() => {
	if (content.value?.kind === 'modpack') return content.value.version_id
	return null
})

const { data: modpackVersion } = useQuery({
	queryKey: computed(() => ['versions', 'detail', modpackVersionId.value]),
	queryFn: () => labrinth.versions_v3.getVersion(modpackVersionId.value!),
	enabled: computed(() => !!modpackVersionId.value),
})

const modpackProjectId = computed(() => modpackVersion.value?.project_id ?? null)

const { data: modpackProject } = useQuery({
	queryKey: computed(() => ['project', 'v3', modpackProjectId.value]),
	queryFn: () => labrinth.projects_v3.get(modpackProjectId.value!),
	enabled: computed(() => !!modpackProjectId.value),
})

const { data: modpackOrg } = useQuery({
	queryKey: computed(() => ['project', 'org', modpackProjectId.value]),
	queryFn: () => labrinth.projects_v3.getOrganization(modpackProjectId.value!),
	enabled: computed(() => !!modpackProjectId.value && !!modpackProject.value?.organization),
})

const usingCustomMrpack = computed(() => modpackVersion.value?.project_id === projectV3.value?.id)

const modpackFileName = computed(() => {
	if (!modpackVersion.value?.files?.length) return null
	const primary = modpackVersion.value.files.find((f) => f.primary)
	return (primary ?? modpackVersion.value.files[0]).filename
})

function handleSetCompatibility() {
	serverCompatibilityModal.value?.show()
}

function handleSwitchCompatibility() {
	serverCompatibilityModal.value?.show({ isSwitchingCompatibilityType: true })
}

function handleUpdateContent() {
	if (!content.value?.kind) return

	switch (content.value.kind) {
		case 'vanilla':
			serverCompatibilityModal.value?.show({ updateContentKind: 'vanilla' })
			break
		case 'modpack':
			if (usingCustomMrpack.value) {
				serverCompatibilityModal.value?.show({ updateContentKind: 'custom-modpack' })
			} else {
				serverCompatibilityModal.value?.show({ updateContentKind: 'published-modpack' })
			}
			break
		default:
			break
	}
}
</script>
