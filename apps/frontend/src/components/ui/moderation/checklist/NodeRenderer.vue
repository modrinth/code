<script lang="ts" setup>
import type {
	BooleanNodeBuilder,
	ButtonNodeBuilder,
	DisplayNodeBuilder,
	DropdownNodeBuilder,
	GroupNodeBuilder,
	IdentifiedNodeBuilder,
	InputNodeBuilder,
	LabeledNodeBuilder,
	NodeBuilder,
	NodeState,
	NodeStateWithChildren,
	ValueNodeBuilder,
} from '@modrinth/moderation'
import {
	evalSegment,
	expandVariables,
	flattenProjectV3Variables,
	flattenProjectVariables,
	flattenStaticVariables,
	getBooleanChildState,
	resolve,
	resolveChildren,
	setMessageProject,
} from '@modrinth/moderation'
import {
	ButtonStyled,
	Checkbox,
	Combobox,
	injectProjectPageContext,
	MarkdownEditor,
	StyledInput,
} from '@modrinth/ui'
import { renderHighlightedString, renderString } from '@modrinth/utils'
import { inject, reactive, watchEffect } from 'vue'

import { NODE_META_KEY, STATE_KEY } from './checklist-context'

const nodeMetaMap = inject(NODE_META_KEY)
const injectedGlobalState = inject(STATE_KEY)

const { projectV3: project, projectV2 } = injectProjectPageContext()
setMessageProject(project, projectV2)

const props = defineProps<{
	nodes: NodeBuilder[]
	showContext: Record<string, NodeState>
	onImageUpload?: (file: File) => Promise<string>
	flex?: boolean
	titleDepth?: number
	parentStatePath?: string[]
}>()

function titleClass(depth: number): string {
	if (depth === 0) return 'text-lg font-extrabold text-contrast'
	if (depth === 1) return 'text-base font-semibold'
	if (depth === 2) return 'text-sm font-semibold'
	return ''
}

function isVisible(node: NodeBuilder): boolean {
	return node._shown === undefined || resolve(node._shown)
}

function isEnabled(node: IdentifiedNodeBuilder): boolean {
	const e = node._enabled
	if (e === undefined) return true
	if (typeof e === 'function') return e(props.showContext)
	return resolve(e)
}

function isButtonEnabled(node: ButtonNodeBuilder): boolean {
	const enabled = node._enabled
	if (enabled === undefined) return true
	if (typeof enabled === 'function') return enabled(props.showContext)
	return resolve(enabled)
}

function asBool(node: NodeBuilder): BooleanNodeBuilder {
	return node as BooleanNodeBuilder
}

function asButton(node: NodeBuilder): ButtonNodeBuilder {
	return node as ButtonNodeBuilder
}

function asIdentified(node: NodeBuilder): IdentifiedNodeBuilder {
	return node as IdentifiedNodeBuilder
}

function asLabeled(node: NodeBuilder): LabeledNodeBuilder {
	return node as LabeledNodeBuilder
}

function asGroup(node: NodeBuilder): GroupNodeBuilder {
	return node as GroupNodeBuilder
}

function asDropdown(node: NodeBuilder): DropdownNodeBuilder {
	return node as DropdownNodeBuilder
}

function asDisplay(node: NodeBuilder): DisplayNodeBuilder {
	return node as DisplayNodeBuilder
}

function asInput(node: NodeBuilder): InputNodeBuilder {
	return node as InputNodeBuilder
}

function getAtPath(path: string[]): NodeState {
	let current: unknown = injectedGlobalState!.value
	for (const key of path) {
		if (current == null || typeof current !== 'object' || current instanceof Set) return undefined
		current = (current as Record<string, unknown>)[key]
	}
	return current as NodeState
}

function setAtPath(path: string[], value: NodeState): void {
	if (path.length === 0) return
	const global = injectedGlobalState!.value as unknown as Record<string, unknown>
	let current = global
	const stack: [Record<string, unknown>, string][] = []
	for (let i = 0; i < path.length - 1; i++) {
		const key = path[i]
		const next = current[key]
		if (!next || typeof next !== 'object' || next instanceof Set) {
			if (value === undefined) return
			current[key] = next !== null && next !== undefined ? { value: next } : {}
			current = current[key] as Record<string, unknown>
		} else {
			stack.push([current, key])
			current = next as Record<string, unknown>
		}
	}
	const lastKey = path[path.length - 1]
	if (value === undefined) {
		Reflect.deleteProperty(current, lastKey)
		for (let i = stack.length - 1; i >= 0; i--) {
			const [parent, key] = stack[i]
			const child = parent[key]
			if (
				child &&
				typeof child === 'object' &&
				!(child instanceof Set) &&
				Object.keys(child as object).length === 0
			) {
				Reflect.deleteProperty(parent, key)
			} else {
				break
			}
		}
	} else {
		current[lastKey] = value as unknown
	}
}

function getNodeState(node: IdentifiedNodeBuilder): NodeState {
	return node._statePath ? getAtPath(node._statePath) : undefined
}

function setNodeState(node: IdentifiedNodeBuilder, value: NodeState): void {
	if (node._statePath) setAtPath(node._statePath, value)
}

function getBooleanState(node: BooleanNodeBuilder): boolean {
	const state = getNodeState(node)
	if (typeof state === 'boolean') return state
	if (state && typeof state === 'object' && !(state instanceof Set)) {
		const v = (state as NodeStateWithChildren).value
		if (typeof v === 'boolean') return v
	}
	const def = resolveDefault(node)
	return (def as boolean | undefined) ?? false
}

function getMultiSelectState(node: IdentifiedNodeBuilder): Set<string> {
	const state = getNodeState(node)
	return state instanceof Set ? state : new Set<string>()
}

function getSelectState(node: IdentifiedNodeBuilder): string | undefined {
	const state = getNodeState(node)
	if (typeof state === 'string') return state
	const def = resolveDefault(node as ValueNodeBuilder)
	return typeof def === 'string' ? def : undefined
}

function getDropdownOptions(node: DropdownNodeBuilder) {
	return [
		...(node._none !== undefined ? [{ value: '', label: node._none }] : []),
		...visibleChildren(node).map((c) => ({
			value: asIdentified(c).id!,
			label: asLabeled(c).label,
		})),
	]
}

function getDropdownModelValue(node: DropdownNodeBuilder) {
	return getSelectState(node) ?? (node._none !== undefined ? '' : undefined)
}

function toggleSelect(parent: IdentifiedNodeBuilder, child: IdentifiedNodeBuilder) {
	const current = getSelectState(parent)
	setNodeState(parent, current === child.id ? undefined : child.id)
}

function resolveDefault(node: ValueNodeBuilder): NodeState {
	const d = node._defaultValue
	return typeof d === 'function' ? d(props.showContext) : d
}

function getTextState(node: IdentifiedNodeBuilder): string {
	const state = getNodeState(node)
	if (typeof state === 'string') return state
	const def = resolveDefault(node as ValueNodeBuilder)
	return typeof def === 'string' ? def : ''
}

function getNodeTitle(node: NodeBuilder): string | undefined {
	if (node._title === undefined) return undefined
	return resolve(node._title) || undefined
}

function getPlaceholder(node: InputNodeBuilder): string | undefined {
	if (node._placeholder !== undefined) return resolve(node._placeholder)
	const def = resolveDefault(node)
	if (typeof def === 'string') return def
	return undefined
}

function hasActionableFixes(node: IdentifiedNodeBuilder): boolean {
	return nodeMetaMap?.value.get(node)?.isFixActionable ?? false
}

function hasRequiredMissingDescendants(node: IdentifiedNodeBuilder): boolean {
	for (const child of getChildren(node)) {
		const identified = child as IdentifiedNodeBuilder
		if (nodeMetaMap?.value.get(identified)?.hasRequiredMissing) return true
		if (identified.id !== undefined && hasRequiredMissingDescendants(identified)) return true
	}
	return false
}

function nodeHasRequiredMissing(node: IdentifiedNodeBuilder): boolean {
	return !!nodeMetaMap?.value.get(node)?.hasRequiredMissing || hasRequiredMissingDescendants(node)
}

function getBooleanColor(node: BooleanNodeBuilder): string {
	if (!getBooleanState(node)) return 'standard'
	if (hasRequiredMissingDescendants(node)) return 'orange'
	return hasActionableFixes(node) ? 'blue' : 'brand'
}

function setTextState(node: IdentifiedNodeBuilder, v: string): void {
	const def = resolveDefault(node as ValueNodeBuilder)
	const defStr = typeof def === 'string' ? def : ''
	setNodeState(node, v === defStr ? undefined : defStr ? v : v || undefined)
}

function toggleBoolean(node: BooleanNodeBuilder) {
	const raw = getNodeState(node)
	const next = !getBooleanState(node)
	const defaultVal = (resolveDefault(node) as boolean | undefined) ?? false
	const isDefault = next === defaultVal
	if (raw && typeof raw === 'object' && !(raw instanceof Set)) {
		const { value: _v, ...children } = raw as NodeStateWithChildren & Record<string, NodeState>
		const hasChildren = Object.keys(children).length > 0
		setNodeState(
			node,
			isDefault && !hasChildren
				? undefined
				: ({ ...children, ...(isDefault ? {} : { value: next }) } as NodeState),
		)
	} else {
		setNodeState(node, isDefault ? undefined : next)
	}
}

function toggleChip(parent: IdentifiedNodeBuilder, child: IdentifiedNodeBuilder) {
	const selected = new Set(getMultiSelectState(parent))
	if (selected.has(child.id!)) {
		selected.delete(child.id!)
		if (child._statePath) {
			setAtPath(child._statePath, undefined)
		}
	} else {
		selected.add(child.id!)
	}
	setNodeState(parent, selected.size > 0 ? selected : undefined)
}

const scopedContextFallbacks = new WeakMap<IdentifiedNodeBuilder, Record<string, NodeState>>()

function childScopedContext(child: IdentifiedNodeBuilder): Record<string, NodeState> {
	if (!child._statePath) return props.showContext
	const basePath = child._statePath
	const state = getAtPath(basePath)
	if (state && typeof state === 'object' && !(state instanceof Set)) {
		return state as Record<string, NodeState>
	}
	const existing = scopedContextFallbacks.get(child)
	if (existing) return existing
	const fallback = new Proxy({} as Record<string, NodeState>, {
		set(_target, key, value) {
			setAtPath([...basePath, key as string], value as NodeState)
			return true
		},
	})
	scopedContextFallbacks.set(child, fallback)
	return fallback
}

function getChildrenContext(node: IdentifiedNodeBuilder): Record<string, NodeState> {
	if (node.type === 'dropdown') return props.showContext
	if (node.type === 'group' && asGroup(node)._selectMode) return props.showContext
	return childScopedContext(node)
}

function getChildren(node: IdentifiedNodeBuilder): NodeBuilder[] {
	return resolveChildren(node, getChildrenContext(node))
}

function visibleChildren(node: IdentifiedNodeBuilder): NodeBuilder[] {
	return getChildren(node).filter(isVisible)
}

function renderProse(content: string): string {
	return renderString(content).replace(/<a /g, '<a target="_blank" ')
}

const tooltipHtml = reactive(new Map<NodeBuilder, string>())

function getNodeTooltipConfig(node: BooleanNodeBuilder) {
	const html = tooltipHtml.get(node)
	if (!html) return undefined
	return {
		content: html,
		html: true,
		delay: { show: 500, hide: 0 },
		triggers: ['hover', 'focus'],
		placement: 'top',
	}
}

const proseContents = reactive(new Map<NodeBuilder, string | null>())

watchEffect(async () => {
	// Read all reactive state synchronously before any await so Vue tracks dependencies
	const asyncDisplayTasks: Array<{ node: DisplayNodeBuilder; state: Record<string, NodeState> }> =
		[]
	const buttonTasks: Array<{ node: BooleanNodeBuilder; state: Record<string, NodeState> }> = []

	for (const node of props.nodes) {
		if (node.type === 'display' && isVisible(node)) {
			asyncDisplayTasks.push({ node: asDisplay(node), state: props.showContext })
		}
		if (node.type === 'toggle' && isVisible(node)) {
			const boolNode = asBool(node)
			if (boolNode._segments.some((s) => s.type !== 'collect')) {
				const nodeState = getNodeState(boolNode)
				const childState =
					nodeState && typeof nodeState === 'object' && !(nodeState instanceof Set)
						? (() => {
								const { value: _v, ...rest } = nodeState as NodeStateWithChildren &
									Record<string, NodeState>
								return rest
							})()
						: {}
				buttonTasks.push({ node: boolNode, state: childState })
			}
		}
		if (node.type === 'group' && asGroup(node)._selectMode === 'multi' && isVisible(node)) {
			for (const child of visibleChildren(asIdentified(node))) {
				const opt = child as IdentifiedNodeBuilder
				if (opt._segments.some((s) => s.type !== 'collect')) {
					const childState = getBooleanChildState(getNodeState(opt)) as Record<string, NodeState>
					buttonTasks.push({ node: opt as BooleanNodeBuilder, state: childState })
				}
			}
		}
	}

	for (const { node, state } of asyncDisplayTasks) {
		proseContents.set(node, await evalSegment(node._segment, state, props.parentStatePath ?? []))
	}

	async function evalCollectedChildren(node: IdentifiedNodeBuilder): Promise<string> {
		let result = ''
		for (const child of getChildren(node)) {
			if (!isVisible(child)) continue
			const childNode = asIdentified(child)
			if (child.type === 'group') {
				const grp = asGroup(childNode)
				if (grp._selectMode === 'multi') {
					const selected = getMultiSelectState(childNode)
					for (const opt of getChildren(childNode)) {
						if (!isVisible(opt)) continue
						const optNode = asIdentified(opt)
						if (!optNode.id || !selected.has(optNode.id)) continue
						result += await evalNodeTooltip(optNode, getBooleanChildState(getNodeState(optNode)) as Record<string, NodeState>)
					}
				} else if (grp._selectMode === 'single') {
					const selected = getSelectState(childNode)
					for (const opt of getChildren(childNode)) {
						if (!isVisible(opt)) continue
						const optNode = asIdentified(opt)
						if (optNode.id !== selected) continue
						result += await evalNodeTooltip(optNode, getBooleanChildState(getNodeState(optNode)) as Record<string, NodeState>)
					}
				} else {
					result += await evalCollectedChildren(childNode)
				}
			} else if (child.type === 'dropdown') {
				const selected = getSelectState(childNode)
				for (const opt of getChildren(childNode)) {
					if (!isVisible(opt)) continue
					const optNode = asIdentified(opt)
					if (optNode.id !== selected) continue
					result += await evalNodeTooltip(optNode, getBooleanChildState(getNodeState(optNode)) as Record<string, NodeState>)
				}
			} else if (child.type === 'toggle' || child.type === 'check') {
				if (!getBooleanState(asBool(childNode))) continue
				result += await evalNodeTooltip(childNode, getBooleanChildState(getNodeState(childNode)) as Record<string, NodeState>)
			}
		}
		return result
	}

	async function evalNodeTooltip(
		node: IdentifiedNodeBuilder,
		state: Record<string, NodeState>,
	): Promise<string> {
		let result = ''
		for (const seg of node._segments) {
			if (seg.type === 'collect') {
				let collected = await evalCollectedChildren(node)
				if (!collected.trim() && seg.fallback) {
					collected = await evalSegment(seg.fallback, state, node._statePath ?? [])
				}
				result += collected
			} else {
				result += await evalSegment(seg, state, node._statePath ?? [])
			}
		}
		return result
	}

	for (const { node, state } of buttonTasks) {
		try {
			const raw = await evalNodeTooltip(node as unknown as IdentifiedNodeBuilder, state)
			const expanded = expandVariables(raw, projectV2.value, project.value, {
				...flattenStaticVariables(),
				...flattenProjectVariables(projectV2.value),
				...flattenProjectV3Variables(project.value),
			})
			const trimmed = expanded.trim()
			tooltipHtml.set(
				node,
				trimmed
					? `<div class="markdown-body moderation-tooltip-markdown">${renderHighlightedString(trimmed)}</div>`
					: '',
			)
		} catch {
			tooltipHtml.set(node, '')
		}
	}
})
</script>

<template>
	<div :class="flex ? 'flex flex-wrap gap-2' : 'space-y-4'">
		<template
			v-for="node in nodes"
			:key="
				node.type === 'button'
					? `button-${asButton(node).label}`
					: (asIdentified(node)._statePath?.join('/') ?? asIdentified(node).id ?? node.type)
			"
		>
			<template v-if="isVisible(node)">
				<div :class="node.type !== 'group' && !getNodeTitle(node) ? 'contents' : undefined">
					<div v-if="getNodeTitle(node)" class="mb-2" :class="titleClass(titleDepth ?? 0)">
						<span
							v-html="renderString(getNodeTitle(node)!).replace(/^<p>([\s\S]*)<\/p>\n?$/, '$1')"
						/><span v-if="nodeHasRequiredMissing(asIdentified(node))" class="text-red">*</span>
					</div>

					<!-- group -->
					<template v-if="node.type === 'group'">
						<!-- multi-select (chips) mode -->
						<template v-if="asGroup(node)._selectMode === 'multi'">
							<div class="flex flex-wrap gap-2">
								<template
									v-for="child in visibleChildren(asIdentified(node))"
									:key="asIdentified(child).id"
								>
									<ButtonStyled
										:color="
											getMultiSelectState(asIdentified(node)).has(asIdentified(child).id!)
												? hasActionableFixes(asIdentified(child))
													? 'blue'
													: 'brand'
												: 'standard'
										"
										@click="toggleChip(asIdentified(node), asIdentified(child))"
									>
										<button
											v-tooltip="getNodeTooltipConfig(asIdentified(child) as BooleanNodeBuilder)"
										>
											{{ asLabeled(child).label }}
										</button>
									</ButtonStyled>
								</template>
							</div>
							<template
								v-for="child in visibleChildren(asIdentified(node))"
								:key="`sub-${asIdentified(child).id}`"
							>
								<NodeRenderer
									v-if="
										getMultiSelectState(asIdentified(node)).has(asIdentified(child).id!) &&
										getChildren(asIdentified(child)).length
									"
									:nodes="getChildren(asIdentified(child))"
									:show-context="getChildrenContext(asIdentified(child))"
									:on-image-upload="onImageUpload"
									:title-depth="titleDepth"
									:parent-state-path="asIdentified(child)._statePath ?? props.parentStatePath ?? []"
									class="mt-2"
								/>
							</template>
						</template>
						<!-- single-select (button-style) mode -->
						<template v-else-if="asGroup(node)._selectMode === 'single'">
							<div class="flex flex-wrap gap-2">
								<template
									v-for="child in visibleChildren(asIdentified(node))"
									:key="asIdentified(child).id"
								>
									<ButtonStyled
										:color="
											getSelectState(asIdentified(node)) === asIdentified(child).id
												? 'brand'
												: 'standard'
										"
									>
										<button @click="toggleSelect(asIdentified(node), asIdentified(child))">
											{{ asLabeled(child).label }}
										</button>
									</ButtonStyled>
								</template>
							</div>
							<template
								v-for="child in visibleChildren(asIdentified(node))"
								:key="`sub-${asIdentified(child).id}`"
							>
								<NodeRenderer
									v-if="
										getSelectState(asIdentified(node)) === asIdentified(child).id &&
										getChildren(asIdentified(child)).length
									"
									:nodes="getChildren(asIdentified(child))"
									:show-context="getChildrenContext(asIdentified(child))"
									:on-image-upload="onImageUpload"
									:title-depth="titleDepth"
									:parent-state-path="asIdentified(child)._statePath ?? props.parentStatePath ?? []"
									class="mt-2"
								/>
							</template>
						</template>
						<!-- plain container mode -->
						<NodeRenderer
							v-else
							:nodes="getChildren(asIdentified(node))"
							:show-context="getChildrenContext(asIdentified(node))"
							:on-image-upload="onImageUpload"
							:flex="asGroup(node)._layout !== 'column'"
							:title-depth="node._title !== undefined ? (titleDepth ?? 0) + 1 : titleDepth"
							:parent-state-path="asIdentified(node)._statePath ?? props.parentStatePath ?? []"
						/>
					</template>

					<!-- dropdown -->
					<template v-else-if="node.type === 'dropdown'">
						<Combobox
							class="!w-80"
							:options="getDropdownOptions(asDropdown(node))"
							:model-value="getDropdownModelValue(asDropdown(node))"
							trigger-class="!bg-[var(--color-button-bg)] !rounded-[var(--radius-md)] !shadow-[var(--shadow-inset-sm),0_0_0_0_transparent]"
							dropdown-class="!rounded-[var(--radius-md)] !bg-[var(--color-button-bg)] !border-0"
							@update:model-value="(v) => setNodeState(asIdentified(node), v || undefined)"
						/>
						<template
							v-for="child in visibleChildren(asIdentified(node))"
							:key="`sub-${asIdentified(child).id}`"
						>
							<NodeRenderer
								v-if="
									getSelectState(asIdentified(node)) === asIdentified(child).id &&
									getChildren(asIdentified(child)).length
								"
								:nodes="getChildren(asIdentified(child))"
								:show-context="getChildrenContext(asIdentified(child))"
								:on-image-upload="onImageUpload"
								:title-depth="titleDepth"
								:parent-state-path="asIdentified(child)._statePath ?? props.parentStatePath ?? []"
								class="mt-2"
							/>
						</template>
					</template>

					<!-- button -->
					<template v-else-if="node.type === 'button'">
						<ButtonStyled>
							<button
								:disabled="!isButtonEnabled(asButton(node))"
								@click="asButton(node)._onClick?.(showContext)"
							>
								{{ asButton(node).label }}
							</button>
						</ButtonStyled>
					</template>

					<!-- toggle -->
					<template v-else-if="node.type === 'toggle'">
						<ButtonStyled :color="getBooleanColor(asBool(node))">
							<button
								v-tooltip="getNodeTooltipConfig(asBool(node))"
								:disabled="!isEnabled(asIdentified(node))"
								@click="toggleBoolean(asBool(node))"
							>
								{{ asLabeled(node).label }}
							</button>
						</ButtonStyled>
					</template>

					<!-- check -->
					<template v-else-if="node.type === 'check'">
						<Checkbox
							:model-value="getBooleanState(asBool(node))"
							:label="asLabeled(node).label"
							:disabled="!isEnabled(asIdentified(node))"
							@update:model-value="isEnabled(asIdentified(node)) && toggleBoolean(asBool(node))"
						/>
					</template>

					<!-- text -->
					<template v-else-if="node.type === 'text'">
						<StyledInput
							:id="`node-${asIdentified(node).id}`"
							:model-value="getTextState(asIdentified(node))"
							:placeholder="getPlaceholder(asInput(node))"
							autocomplete="off"
							@update:model-value="(v: string) => setTextState(asIdentified(node), v)"
						/>
					</template>

					<!-- markdown -->
					<template v-else-if="node.type === 'markdown'">
						<MarkdownEditor
							:id="`node-${asIdentified(node).id}`"
							:aria-label="asLabeled(node).label || undefined"
							:model-value="getTextState(asIdentified(node))"
							:placeholder="getPlaceholder(asInput(node))"
							:max-height="300"
							:disabled="false"
							:heading-buttons="false"
							:on-image-upload="onImageUpload"
							@update:model-value="(v: string) => setTextState(asIdentified(node), v)"
						/>
					</template>

					<!-- display (prose + label) -->
					<template v-else-if="node.type === 'display' && proseContents.get(node) != null">
						<div class="markdown-body" v-html="renderProse(proseContents.get(node)!)" />
					</template>
				</div>
			</template>
		</template>

		<!-- children of active boolean nodes, rendered after all siblings -->
		<template
			v-for="node in nodes"
			:key="
				node.type === 'button'
					? `children-button-${asButton(node).label}`
					: `children-${asIdentified(node)._statePath?.join('/') ?? asIdentified(node).id ?? node.type}`
			"
		>
			<NodeRenderer
				v-if="
					isVisible(node) &&
					(node.type === 'toggle' || node.type === 'check') &&
					getBooleanState(asBool(node)) &&
					getChildren(asIdentified(node)).length
				"
				:nodes="getChildren(asIdentified(node))"
				:show-context="getChildrenContext(asIdentified(node))"
				:on-image-upload="onImageUpload"
				:title-depth="node._title !== undefined ? (titleDepth ?? 0) + 1 : titleDepth"
				:parent-state-path="asIdentified(node)._statePath ?? props.parentStatePath ?? []"
				class="w-full"
			/>
		</template>
	</div>
</template>
