<template>
	<div class="flex w-full max-w-full flex-col gap-3">
		<div class="grid gap-2.5">
			<span class="font-semibold text-contrast">Project</span>
			<DependencySelect v-model="newDependencyProjectId" />
		</div>

		<template v-if="newDependencyProjectId">
			<div class="grid gap-2.5">
				<span class="font-semibold text-contrast"> Version </span>
				<Combobox
					v-model="newDependencyVersionId"
					placeholder="Select version"
					:options="newDependencyVersionOptions"
					:search-value="selectedNewDependencyVersionLabel"
					:searchable="true"
					:select-search-text-on-focus="true"
				>
					<template #option="{ item, isSelected }">
						<div class="flex w-full items-center justify-between gap-2">
							<a
								v-if="item.value"
								:href="getDependencyVersionUrl(item.value)"
								target="_blank"
								rel="noopener noreferrer"
								class="custom-focus-indicator flex min-w-0 items-center gap-1 rounded-sm font-semibold leading-tight outline-none hover:underline focus-visible:underline"
								:class="isSelected ? 'text-green' : 'text-primary'"
								:aria-label="`Open ${item.label} in a new tab`"
								@mousedown.stop
								@click.stop
							>
								<span class="truncate">{{ item.label }}</span>
								<ExternalIcon
									aria-hidden="true"
									class="size-4 shrink-0 opacity-0 transition-opacity group-hover/option:opacity-100 group-data-[focused=true]/option:opacity-100"
								/>
							</a>
							<span
								v-else
								class="font-semibold leading-tight"
								:class="isSelected ? 'text-green' : 'text-primary'"
							>
								{{ item.label }}
							</span>
							<div
								v-if="getDependencyVersionOption(item.value)"
								class="flex flex-wrap items-center justify-end gap-1.5"
							>
								<TagItem
									v-for="platform in getDependencyVersionOption(item.value)?.platforms.slice(
										0,
										MAX_VISIBLE_TAGS,
									)"
									:key="`platform-${platform}`"
									class="shrink-0 border !border-solid border-surface-5"
									:style="`--_color: var(--color-platform-${platform})`"
								>
									<FormattedTag :tag="platform" enforce-type="loader" />
								</TagItem>
								<TagsOverflow
									:tags="
										getDependencyVersionOption(item.value)?.platforms.slice(MAX_VISIBLE_TAGS) ?? []
									"
									class="shrink-0 border !border-solid border-surface-5"
								/>
								<TagItem
									v-for="gameVersion in getDependencyVersionOption(item.value)?.gameVersions.slice(
										0,
										MAX_VISIBLE_TAGS,
									)"
									:key="`game-version-${gameVersion}`"
									class="shrink-0 border !border-solid border-surface-5"
								>
									{{ gameVersion }}
								</TagItem>
								<TagsOverflow
									:tags="
										getDependencyVersionOption(item.value)?.gameVersions.slice(MAX_VISIBLE_TAGS) ??
										[]
									"
									class="shrink-0 border !border-solid border-surface-5"
								/>
							</div>
						</div>
					</template>
				</Combobox>
			</div>

			<div class="grid gap-2.5">
				<span class="font-semibold text-contrast"> Dependency relation </span>
				<Combobox
					v-model="newDependencyType"
					placeholder="Select dependency type"
					:options="[
						{ label: 'Required', value: 'required' },
						{ label: 'Optional', value: 'optional' },
						{ label: 'Incompatible', value: 'incompatible' },
						{ label: 'Embedded', value: 'embedded' },
					]"
				/>
			</div>
		</template>
	</div>
</template>

<script lang="ts" setup>
import { ExternalIcon } from '@modrinth/assets'
import { Combobox, FormattedTag, TagItem, TagsOverflow } from '@modrinth/ui'
import { computed } from 'vue'

import DependencySelect from '~/components/ui/create-project-version/components/DependencySelect.vue'
import {
	type DependencyVersionOption,
	injectManageVersionContext,
} from '~/providers/version/manage-version-modal'

const { newDependencyProjectId, newDependencyType, newDependencyVersionId, newDependencyVersions } =
	injectManageVersionContext()

const MAX_VISIBLE_TAGS = 2

const newDependencyVersionOptions = computed(() => [
	{ label: 'Any version', value: null },
	...newDependencyVersions.value,
])
const selectedNewDependencyVersionLabel = computed(
	() =>
		newDependencyVersionOptions.value.find(
			(option) => option.value === newDependencyVersionId.value,
		)?.label,
)

const getDependencyVersionOption = (
	versionId: string | null,
): DependencyVersionOption | undefined =>
	newDependencyVersions.value.find((option) => option.value === versionId)

const getDependencyVersionUrl = (versionId: string): string =>
	`/project/${encodeURIComponent(newDependencyProjectId.value ?? '')}/version/${encodeURIComponent(versionId)}`
</script>
