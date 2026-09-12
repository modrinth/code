<template>
	<div
		v-if="!barOnly || hasStages"
		class="flex min-h-0 min-w-0 flex-col"
		:class="barOnly ? '' : 'flex-1'"
		@contextmenu="onContextMenu"
	>
		<!-- Auto button bars: one per checklist stage mapped to this element -->
		<div
			v-if="autoBars && hasStages"
			class="flex shrink-0 flex-col gap-1.5 border-0 border-solid border-divider p-2"
			:class="barOnly ? '' : 'border-b bg-surface-1'"
		>
			<ChecklistStageButtons v-for="s in stages" :key="s.id" :stage-id="s.id" variant="inline" />
		</div>

		<div v-if="!barOnly" class="min-h-0 min-w-0 flex-1 overflow-y-auto">
			<slot />
		</div>

		<!-- Right-click quick menu -->
		<template v-if="menu">
			<div class="fixed inset-0 z-[60]" @click="menu = null" @contextmenu.prevent="menu = null" />
			<div
				class="fixed z-[61] flex max-h-[70vh] min-w-[15rem] flex-col overflow-y-auto rounded-lg border border-solid border-divider bg-bg-raised p-1 text-sm shadow-lg"
				:style="{ left: `${menu.x}px`, top: `${menu.y}px` }"
			>
				<template v-for="grp in menuGroups" :key="grp.key">
					<p class="m-0 px-2 pb-0.5 pt-1.5 text-xs font-semibold uppercase text-secondary">
						{{ grp.label }}
					</p>
					<button
						v-for="opt in grp.options"
						:key="opt.key"
						class="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-primary hover:bg-button-bg"
						@click="toggleOption(opt)"
					>
						<CheckIcon class="size-4 shrink-0" :class="opt.active ? 'text-green' : 'opacity-25'" />
						{{ opt.label }}
					</button>
				</template>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import type { NodeState, StageNode } from '@modrinth/moderation/src/types/node'
import {
	getBooleanChildState,
	hasChildrenCap,
	hasIdCap,
	hasValueCap,
	isShown,
	resolve,
	resolveChildren,
} from '@modrinth/moderation/src/types/node'
import { computed, ref } from 'vue'

import {
	type ChecklistElementKey,
	injectModerationChecklist,
} from '~/components/ui/moderation/checklist/checklist-context'

import ChecklistStageButtons from './ChecklistStageButtons.vue'

const props = withDefaults(
	defineProps<{
		elementKey?: ChecklistElementKey | null
		/** Render only the button bars (no slot / scroll container) — for sidebar cards. */
		barOnly?: boolean
		/** When false, the host places `ChecklistStageButtons` itself (e.g. the Description tab). */
		autoBars?: boolean
	}>(),
	{ elementKey: null, barOnly: false, autoBars: true },
)

const engine = injectModerationChecklist()
const settings = useModerationSettings()

const stages = computed<StageNode[]>(() =>
	props.elementKey ? engine.stagesForElement(props.elementKey) : [],
)
const hasStages = computed(() => stages.value.length > 0)

/* --- Right-click quick menu -------------------------------------------------- */

interface MenuOption {
	key: string
	label: string
	statePath: string[]
	active: boolean
}
interface MenuGroup {
	key: string
	label: string
	options: MenuOption[]
}

function groupTitle(node: object): string | null {
	if (!('_title' in node)) return null
	const t = (node as { _title?: unknown })._title
	return t === undefined ? null : resolve(t as never) || null
}

/** Flatten each mapped stage to `{ section → toggle options }` for the context menu. */
const menuGroups = computed<MenuGroup[]>(() => {
	const groups: MenuGroup[] = []

	for (const stage of stages.value) {
		const buckets = new Map<string, MenuOption[]>()
		const order: string[] = []

		const walk = (
			nodes: unknown[],
			scope: Record<string, NodeState>,
			path: string[],
			section: string,
		) => {
			for (const node of nodes) {
				if (typeof node !== 'object' || node === null || !isShown(node as never)) continue

				if (hasValueCap(node) && hasIdCap(node) && 'label' in node) {
					const id = (node as { id: string }).id
					const np = [...path, id]
					if (!buckets.has(section)) {
						buckets.set(section, [])
						order.push(section)
					}
					buckets.get(section)!.push({
						key: np.join('/'),
						label: String((node as { label: unknown }).label),
						statePath: np,
						active: scope[id] === true,
					})
					if (hasChildrenCap(node)) {
						const childScope = getBooleanChildState(scope[id])
						walk(resolveChildren(node as never, childScope), childScope, np, section)
					}
				} else if (hasChildrenCap(node)) {
					const nested =
						hasIdCap(node) &&
						typeof scope[(node as { id: string }).id] === 'object' &&
						!(scope[(node as { id: string }).id] instanceof Set)
					const childScope = nested
						? (scope[(node as { id: string }).id] as Record<string, NodeState>)
						: scope
					const childPath = hasIdCap(node) ? [...path, (node as { id: string }).id] : path
					walk(
						resolveChildren(node as never, childScope),
						childScope,
						childPath,
						groupTitle(node) || section,
					)
				}
			}
		}

		const stageScope = (engine.nodeStates.value[stage.id] ?? {}) as Record<string, NodeState>
		walk(resolveChildren(stage, stageScope), stageScope, [stage.id], '')

		for (const section of order) {
			groups.push({
				key: `${stage.id}::${section}`,
				label: section || stage.label || stage.id,
				options: buckets.get(section)!,
			})
		}
	}

	return groups
})

function toggleOption(opt: MenuOption) {
	engine.setNodeActive(opt.statePath, !opt.active)
}

const menu = ref<{ x: number; y: number } | null>(null)

function onContextMenu(event: MouseEvent) {
	if (menuGroups.value.length === 0) return
	if (settings.value.get(moderationSettings.Experimental.InlineChecklistMenu) !== true) return
	event.preventDefault()
	event.stopPropagation()
	const view = (event.view as Window | null) ?? window
	menu.value = {
		x: Math.max(4, Math.min(event.clientX, view.innerWidth - 260)),
		y: Math.max(4, Math.min(event.clientY, view.innerHeight - 340)),
	}
}
</script>
