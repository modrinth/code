<template>
	<div v-if="dependencies.length > 0" class="flex flex-col gap-2">
		<h3 class="m-0 text-sm font-bold text-contrast">
			{{ formatMessage(messages.dependenciesTitle) }}
		</h3>
		<div class="flex flex-col gap-2">
			<div v-for="dependency in dependencies" :key="dependency.key" class="flex flex-col gap-1.5">
				<div
					class="grid min-h-9 grid-cols-[minmax(0,1fr)_min-content] items-center gap-2 rounded-xl bg-button-bg px-3 py-2 text-primary"
				>
					<span class="flex min-w-0 items-center gap-2">
						<Avatar v-if="dependency.icon" :src="dependency.icon" :alt="dependency.name" size="20px" />
						<PackageIcon v-else aria-hidden="true" class="size-5 flex-shrink-0 text-secondary" />
						<a
							v-if="dependency.projectHref"
							:href="dependency.projectHref"
							target="_blank"
							rel="noopener noreferrer"
							class="min-w-0 truncate font-semibold text-contrast no-underline hover:underline"
						>
							{{ dependency.name }}
						</a>
						<span v-else class="min-w-0 truncate font-semibold text-contrast">
							{{ dependency.name }}
						</span>
						<TagItem class="shrink-0 border !border-solid border-surface-5">
							{{ dependency.typeLabel }}
						</TagItem>
					</span>
					<ButtonStyled v-if="dependency.downloadHref" circular type="transparent">
						<a
							v-tooltip="'Download'"
							:href="dependency.downloadHref"
							:download="dependency.filename"
							:aria-label="`Download ${dependency.name}`"
							@click="emit('download')"
						>
							<DownloadIcon aria-hidden="true" class="size-5 text-secondary" />
						</a>
					</ButtonStyled>
				</div>
				<div
					v-for="subDependency in dependency.subDependencies"
					:key="subDependency.key"
					class="grid grid-cols-[1.5rem_minmax(0,1fr)] items-center gap-1 pl-5"
				>
					<RightArrowIcon aria-hidden="true" class="size-4 text-secondary" />
					<div
						class="grid min-h-9 grid-cols-[minmax(0,1fr)_min-content] items-center gap-2 rounded-xl bg-button-bg px-3 py-2 text-primary"
					>
						<span class="flex min-w-0 items-center gap-2">
							<Avatar
								v-if="subDependency.icon"
								:src="subDependency.icon"
								:alt="subDependency.name"
								size="20px"
							/>
							<PackageIcon v-else aria-hidden="true" class="size-5 flex-shrink-0 text-secondary" />
							<a
								v-if="subDependency.projectHref"
								:href="subDependency.projectHref"
								target="_blank"
								rel="noopener noreferrer"
								class="min-w-0 truncate font-semibold text-contrast no-underline hover:underline"
							>
								{{ subDependency.name }}
							</a>
							<span v-else class="min-w-0 truncate font-semibold text-contrast">
								{{ subDependency.name }}
							</span>
							<TagItem class="shrink-0 border !border-solid border-surface-5">
								{{ subDependency.typeLabel }}
							</TagItem>
						</span>
						<ButtonStyled v-if="subDependency.downloadHref" circular type="transparent">
							<a
								v-tooltip="'Download'"
								:href="subDependency.downloadHref"
								:download="subDependency.filename"
								:aria-label="`Download ${subDependency.name}`"
								@click="emit('download')"
							>
								<DownloadIcon aria-hidden="true" class="size-5 text-secondary" />
							</a>
						</ButtonStyled>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup>
import { DownloadIcon, PackageIcon, RightArrowIcon } from '@modrinth/assets'
import { Avatar, ButtonStyled, defineMessages, TagItem, useVIntl } from '@modrinth/ui'

defineOptions({
	name: 'DownloadDependenciesSection',
})

defineProps({
	dependencies: {
		type: Array,
		default: () => [],
	},
})

const emit = defineEmits(['download'])
const { formatMessage } = useVIntl()

const messages = defineMessages({
	dependenciesTitle: {
		id: 'project.download.dependencies-title',
		defaultMessage: 'Dependencies',
	},
})
</script>
