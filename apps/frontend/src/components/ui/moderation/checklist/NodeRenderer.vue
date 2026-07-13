<script lang="ts" setup>
import type {
	BooleanNodeBuilder,
	ButtonNodeBuilder,
	ContentFn,
	DisplayNodeBuilder,
	GroupNodeBuilder,
	IdentifiedNodeBuilder,
	InputNodeBuilder,
	NodeBuilder,
	NodeContext,
	NodeState,
	NodeStateWithChildren,
	SelectNodeBuilder,
	ValueNodeBuilder,
} from '@modrinth/moderation'
import {
	expandVariables,
	flattenProjectV3Variables,
	flattenProjectVariables,
	flattenStaticVariables,
	resolveChildren,
} from '@modrinth/moderation'
import { ButtonStyled, Checkbox, Combobox, MarkdownEditor, StyledInput } from '@modrinth/ui'
import { renderHighlightedString, renderString } from '@modrinth/utils'
import { inject, reactive, watchEffect } from 'vue'

import { NODE_META_KEY } from './checklist-context'

const nodeMetaMap = inject(NODE_META_KEY)

const props = defineProps<{
	nodes: NodeBuilder[]
	showContext: NodeContext
	onImageUpload?: (file: File) => Promise<string>
	flex?: boolean
}>()

function isVisible(node: NodeBuilder): boolean {
	return !node._shown || node._shown(props.showContext)
}

function isEnabled(node: IdentifiedNodeBuilder): boolean {
	return !node._enabled || node._enabled(props.showContext)
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

function asGroup(node: NodeBuilder): GroupNodeBuilder {
	return node as GroupNodeBuilder
}

function asDisplay(node: NodeBuilder): DisplayNodeBuilder {
	return node as DisplayNodeBuilder
}

function asValue(node: NodeBuilder): ValueNodeBuilder {
	return node as ValueNodeBuilder
}

function asInput(node: NodeBuilder): InputNodeBuilder {
	return node as InputNodeBuilder
}

function asSelect(node: NodeBuilder): SelectNodeBuilder {
	return node as SelectNodeBuilder
}

function getAtPath(path: string[]): NodeState {
	let current: unknown = props.showContext.globalState
	for (const key of path) {
		if (current == null || typeof current !== 'object' || current instanceof Set) return undefined
		current = (current as Record<string, unknown>)[key]
	}
	return current as NodeState
}

function setAtPath(path: string[], value: NodeState): void {
	if (path.length === 0) return
	const global = props.showContext.globalState as unknown as Record<string, unknown>
	let current = global
	for (let i = 0; i < path.length - 1; i++) {
		const key = path[i]
		const next = current[key]
		if (!next || typeof next !== 'object' || next instanceof Set) {
			current[key] = (next !== null && next !== undefined) ? { value: next } : {}
			current = current[key] as Record<string, unknown>
		} else {
			current = next as Record<string, unknown>
		}
	}
	const lastKey = path[path.length - 1]
	if (value === undefined) {
		delete current[lastKey]
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

function getDropdownOptions(node: SelectNodeBuilder) {
	const none = typeof node._dropdown === 'object' ? node._dropdown.none : undefined
	return [
		...(none !== undefined ? [{ value: '', label: none }] : []),
		...visibleChildren(node).map((c) => ({ value: asIdentified(c).id!, label: asIdentified(c).label })),
	]
}

function getDropdownModelValue(node: SelectNodeBuilder) {
	const none = typeof node._dropdown === 'object' ? node._dropdown.none : undefined
	return getSelectState(node) ?? (none !== undefined ? '' : undefined)
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

function getPlaceholder(node: InputNodeBuilder): string | undefined {
	if (node._placeholder) return node._placeholder
	const def = resolveDefault(node)
	if (typeof def === 'string') return def
	return asIdentified(node).label || undefined
}

function hasActionableFixes(node: IdentifiedNodeBuilder): boolean {
	return nodeMetaMap?.value.get(node)?.isFixActionable ?? false
}

function getBooleanColor(node: BooleanNodeBuilder): string {
	if (!getBooleanState(node)) return 'standard'
	return hasActionableFixes(node) ? 'blue' : 'brand'
}

function setTextState(node: IdentifiedNodeBuilder, v: string): void {
	const def = resolveDefault(node as ValueNodeBuilder)
	const defStr = typeof def === 'string' ? def : ''
	setNodeState(node, v === defStr ? undefined : (defStr ? v : (v || undefined)))
}

function toggleBoolean(node: BooleanNodeBuilder) {
	const raw = getNodeState(node)
	const next = !getBooleanState(node)
	const defaultVal = (resolveDefault(node) as boolean | undefined) ?? false
	const isDefault = next === defaultVal
	if (raw && typeof raw === 'object' && !(raw instanceof Set)) {
		const { value: _v, ...children } = raw as NodeStateWithChildren & Record<string, NodeState>
		const hasChildren = Object.keys(children).length > 0
		setNodeState(node, isDefault && !hasChildren ? undefined : { ...children, ...(isDefault ? {} : { value: next }) } as NodeState)
	} else {
		setNodeState(node, isDefault ? undefined : next)
	}
}

function toggleChip(parent: IdentifiedNodeBuilder, child: IdentifiedNodeBuilder) {
	const selected = new Set(getMultiSelectState(parent))
	if (selected.has(child.id!)) {
		selected.delete(child.id!)
	} else {
		selected.add(child.id!)
	}
	setNodeState(parent, selected.size > 0 ? selected : undefined)
}

function getChildren(node: IdentifiedNodeBuilder): NodeBuilder[] {
	return resolveChildren(node, props.showContext)
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
	const asyncDisplayTasks: Array<{ node: DisplayNodeBuilder; ctx: NodeContext }> = []
	const buttonTasks: Array<{ node: BooleanNodeBuilder; ctx: NodeContext }> = []

	for (const node of props.nodes) {
		if (node.type === 'display' && isVisible(node)) {
			const display = asDisplay(node)
			if (typeof display._content === 'string') {
				proseContents.set(node, display._content)
			} else {
				asyncDisplayTasks.push({ node: display, ctx: props.showContext })
			}
		}
		if (node.type === 'toggle' && isVisible(node)) {
			const boolNode = asBool(node)
			if (boolNode._action?._message) {
				const nodeState = getNodeState(boolNode)
				const childState = (nodeState && typeof nodeState === 'object' && !(nodeState instanceof Set))
					? (() => { const { value: _v, ...rest } = nodeState as NodeStateWithChildren & Record<string, NodeState>; return rest })()
					: {}
				buttonTasks.push({ node: boolNode, ctx: { ...props.showContext, state: childState } })
			}
		}
	}

	for (const { node, ctx } of asyncDisplayTasks) {
		proseContents.set(node, await (node._content as ContentFn)(ctx))
	}

	for (const { node, ctx } of buttonTasks) {
		try {
			const raw = await node._action!._message!(ctx)
			const expanded = expandVariables(raw, props.showContext.projectV2, props.showContext.project, {
				...flattenStaticVariables(),
				...flattenProjectVariables(props.showContext.projectV2),
				...flattenProjectV3Variables(props.showContext.project),
			})
			const trimmed = expanded.trim()
			tooltipHtml.set(node, trimmed
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
		<template v-for="node in nodes" :key="node.type === 'button' ? `button-${asButton(node).label}` : (asIdentified(node).id ?? node.type)">
			<template v-if="isVisible(node)">
				<!-- group -->
				<div v-if="node.type === 'group'">
					<div v-if="asGroup(node)._title" class="mb-2 font-semibold">
						{{ asGroup(node)._title }}<span v-if="asGroup(node)._required" class="text-red"> *</span>
					</div>
					<NodeRenderer
						:nodes="getChildren(asIdentified(node))"
						:show-context="showContext"
						:on-image-upload="onImageUpload"
						:flex="asGroup(node)._layout !== 'column'"
					/>
				</div>

				<!-- button -->
				<ButtonStyled v-else-if="node.type === 'button'">
					<button
						:disabled="asButton(node)._enabled ? !asButton(node)._enabled(showContext) : false"
						@click="asButton(node)._onClick?.(showContext)"
					>{{ asButton(node).label }}</button>
				</ButtonStyled>

				<!-- toggle -->
				<ButtonStyled
					v-else-if="node.type === 'toggle'"
					:color="getBooleanColor(asBool(node))"
				>
					<button
						v-tooltip="getNodeTooltipConfig(asBool(node))"
						:disabled="!isEnabled(asIdentified(node))"
						@click="toggleBoolean(asBool(node))"
					>{{ asIdentified(node).label }}</button>
				</ButtonStyled>

				<!-- check -->
				<Checkbox
					v-else-if="node.type === 'check'"
					:model-value="getBooleanState(asBool(node))"
					:label="asIdentified(node).label"
					:disabled="!isEnabled(asIdentified(node))"
					@update:model-value="isEnabled(asIdentified(node)) && toggleBoolean(asBool(node))"
				/>

				<!-- select (single-choice) -->
				<div v-else-if="node.type === 'select'">
					<div class="mb-2 font-semibold">{{ asIdentified(node).label }}<span v-if="asValue(node)._required" class="text-red"> *</span></div>
					<Combobox
						v-if="asSelect(node)._dropdown !== false"
						:class="asSelect(node)._fullWidth ? 'w-full' : '!w-80'"
						:options="getDropdownOptions(asSelect(node))"
						:model-value="getDropdownModelValue(asSelect(node))"
						trigger-class="!bg-[var(--color-button-bg)] !rounded-[var(--radius-md)] !shadow-[var(--shadow-inset-sm),0_0_0_0_transparent]"
						dropdown-class="!rounded-[var(--radius-md)] !bg-[var(--color-button-bg)] !border-0"
						@update:model-value="(v) => setNodeState(asIdentified(node), v || undefined)"
					/>
					<div v-else class="flex flex-wrap gap-2">
						<template v-for="child in visibleChildren(asIdentified(node))" :key="asIdentified(child).id">
							<ButtonStyled :color="getSelectState(asIdentified(node)) === asIdentified(child).id ? 'brand' : 'standard'">
								<button
									@click="toggleSelect(asIdentified(node), asIdentified(child))"
								>{{ asIdentified(child).label }}</button>
							</ButtonStyled>
						</template>
					</div>
					<template v-for="child in visibleChildren(asIdentified(node))" :key="`sub-${asIdentified(child).id}`">
						<NodeRenderer
							v-if="getSelectState(asIdentified(node)) === asIdentified(child).id && getChildren(asIdentified(child)).length"
							:nodes="getChildren(asIdentified(child))"
							:show-context="showContext"
							:on-image-upload="onImageUpload"
							class="mt-2"
						/>
					</template>
				</div>

				<!-- multi-select (chips) -->
				<div v-else-if="node.type === 'multi-select'">
					<div class="mb-2 font-semibold">{{ asIdentified(node).label }}<span v-if="asValue(node)._required" class="text-red"> *</span></div>
					<div class="flex flex-wrap gap-2">
						<template v-for="child in visibleChildren(asIdentified(node))" :key="asIdentified(child).id">
							<ButtonStyled
								:color="getMultiSelectState(asIdentified(node)).has(asIdentified(child).id!) ? (hasActionableFixes(asIdentified(child)) ? 'blue' : 'brand') : 'standard'"
								@click="toggleChip(asIdentified(node), asIdentified(child))"
							>
								<button>{{ asIdentified(child).label }}</button>
							</ButtonStyled>
						</template>
					</div>
					<template v-for="child in visibleChildren(asIdentified(node))" :key="`sub-${asIdentified(child).id}`">
						<NodeRenderer
							v-if="getMultiSelectState(asIdentified(node)).has(asIdentified(child).id!) && getChildren(asIdentified(child)).length"
							:nodes="getChildren(asIdentified(child))"
							:show-context="showContext"
							:on-image-upload="onImageUpload"
							class="mt-2"
						/>
					</template>
				</div>

				<!-- text -->
				<StyledInput
					v-else-if="node.type === 'text'"
					:id="`node-${asIdentified(node).id}`"
					:aria-label="asIdentified(node).label || undefined"
					:model-value="getTextState(asIdentified(node))"
					:placeholder="getPlaceholder(asInput(node))"
					autocomplete="off"
					@update:model-value="(v: string) => setTextState(asIdentified(node), v)"
				/>

				<!-- markdown -->
				<MarkdownEditor
					v-else-if="node.type === 'markdown'"
					:id="`node-${asIdentified(node).id}`"
					:aria-label="asIdentified(node).label || undefined"
					:model-value="getTextState(asIdentified(node))"
					:placeholder="getPlaceholder(asInput(node))"
					:max-height="300"
					:disabled="false"
					:heading-buttons="false"
					:on-image-upload="onImageUpload"
					@update:model-value="(v: string) => setTextState(asIdentified(node), v)"
				/>

				<!-- display (prose + label) -->
				<div
					v-else-if="node.type === 'display' && proseContents.get(node) != null"
					class="markdown-body"
					v-html="renderProse(proseContents.get(node)!)"
				/>
			</template>
		</template>

		<!-- children of active boolean nodes, rendered after all siblings -->
		<template v-for="node in nodes" :key="node.type === 'button' ? `children-button-${asButton(node).label}` : `children-${asIdentified(node).id ?? node.type}`">
			<NodeRenderer
				v-if="isVisible(node) && (node.type === 'toggle' || node.type === 'check') && getBooleanState(asBool(node)) && getChildren(asIdentified(node)).length"
				:nodes="getChildren(asIdentified(node))"
				:show-context="showContext"
				:on-image-upload="onImageUpload"
				class="w-full"
			/>
		</template>
	</div>
</template>
