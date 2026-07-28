<script lang="ts" setup>
import { ButtonStyled } from '@modrinth/ui'
import { renderString } from '@modrinth/utils'
import { inject } from 'vue'
import type { Component } from 'vue'

import type { AnyNode, ChildNode, HasChildren } from '../builder'
import type { ComponentNodePropsContext, Enableable, HasValue, Identified, OnChangeFn } from '../capabilities'
import { CHECKLIST_META_KEY } from '../context'
import { childWriter, writeNodeValue } from '../mutate'
import type { Writer } from '../mutate'
import {
	getBooleanChildState,
	getEffectiveValue,
	hasCap,
	hasChildrenCap,
	hasIdCap,
	hasOptionsCap,
	hasValueCap,
	isNodeActive,
	isShown,
	resolveChildren,
} from '../resolve'
import type { NodeState, Reactive } from '../state'
import { resolve } from '../state'

const metaCtx = inject(CHECKLIST_META_KEY)

const props = defineProps<{
	nodes: ChildNode[]
	state: Record<string, NodeState>
	write: Writer
	onImageUpload?: (file: File) => Promise<string>
	flex?: boolean
	titleDepth?: number
	appComponents?: Record<string, Component>
}>()

type RenderableValueNode = AnyNode &
	HasValue &
	Identified &
	Partial<Enableable> & {
		_component: Component | undefined
		_rendererKey: string | undefined
		_modelProp: string
		_componentProps?: (ctx: ComponentNodePropsContext) => Record<string, unknown>
		_extraProps?: (ctx: ComponentNodePropsContext) => Record<string, unknown>
	}

function resolveComponent(node: RenderableValueNode): Component | undefined {
	return node._component ?? (node._rendererKey ? props.appComponents?.[node._rendererKey] : undefined)
}

function titleClass(depth: number): string {
	if (depth === 0) return 'text-lg font-extrabold text-contrast'
	if (depth === 1) return 'text-base font-semibold'
	if (depth === 2) return 'text-sm font-semibold'
	return ''
}

function getTitle(node: object): string | undefined {
	if (!hasCap(node, '_title')) return undefined
	const title = node._title as Reactive<string> | undefined
	if (title === undefined) return undefined
	return resolve(title) || undefined
}

function needsAttention(node: object): boolean {
	return metaCtx?.value.attentionMap.get(node) ?? false
}

function isFixActionable(node: object): boolean {
	return metaCtx?.value.metaMap.get(node)?.isFixActionable ?? false
}

function isEnabled(node: Partial<Enableable>): boolean {
	if (node._enabled === undefined) return true
	if (typeof node._enabled === 'function') return node._enabled(props.state)
	return resolve(node._enabled)
}

function toggleSetValue(node: RenderableValueNode, value: string): void {
	const current = getEffectiveValue(node, props.state[node.id], props.state) as unknown as string[]
	const set = new Set(Array.isArray(current) ? current : [])
	if (set.has(value)) set.delete(value)
	else set.add(value)
	writeNodeValue(node, props.state, props.write, Array.from(set) as never)
}

const DROPDOWN_TRIGGER_CHROME_PX = 16 * 2 + 10 + 20 + 2
let measureEl: HTMLSpanElement | null = null
function measureLabelWidth(label: string): number {
	if (typeof document === 'undefined') return 0
	if (!measureEl) {
		measureEl = document.createElement('span')
		measureEl.className = 'min-w-0 truncate text-primary font-semibold leading-tight'
		Object.assign(measureEl.style, {
			position: 'absolute',
			visibility: 'hidden',
			whiteSpace: 'nowrap',
			left: '-9999px',
			top: '0',
		})
		document.body.appendChild(measureEl)
	}
	measureEl.textContent = label
	return measureEl.getBoundingClientRect().width
}

const dropdownMinWidthCache = new Map<string, string>()
function getDropdownMinWidth(options: { label: string }[]): string {
	const key = options.map((o) => o.label).join(' ')
	const cached = dropdownMinWidthCache.get(key)
	if (cached) return cached
	const maxLabelWidth = Math.max(0, ...options.map((o) => measureLabelWidth(o.label)))
	const result = `${Math.ceil(maxLabelWidth) + DROPDOWN_TRIGGER_CHROME_PX}px`
	dropdownMinWidthCache.set(key, result)
	return result
}

function componentProps(node: RenderableValueNode): Record<string, unknown> {
	const ctx: ComponentNodePropsContext = {
		onImageUpload: props.onImageUpload,
		toggleSetValue: (value) => toggleSetValue(node, value),
		nodeFacts: { needsAttention: needsAttention(node), fixActionable: isFixActionable(node) },
	}
	const dropdownStyle = hasOptionsCap(node)
		? {
				class: '!w-auto max-w-full',
				style: { minWidth: getDropdownMinWidth(node._options as unknown as { label: string }[]) },
			}
		: undefined
	return {
		disabled: !isEnabled(node),
		...dropdownStyle,
		...node._componentProps?.(ctx),
		...node._extraProps?.(ctx),
	}
}

function containerScope(node: HasChildren & Partial<Identified>): {
	state: Record<string, NodeState>
	write: Writer
} {
	if (!hasIdCap(node)) return { state: props.state, write: props.write }
	const raw = props.state[node.id]
	const state = raw && typeof raw === 'object' && !(raw instanceof Set) ? (raw as Record<string, NodeState>) : {}
	return { state, write: childWriter(props.state, props.write, node.id) }
}

function valueScope(node: HasValue & Identified): { state: Record<string, NodeState>; write: Writer } {
	const state = getBooleanChildState(props.state[node.id])
	return { state, write: childWriter(props.state, props.write, node.id) }
}

function resolveTooltip(node: object): { content: string } | undefined {
	if (!hasCap(node, '_tooltip')) return undefined
	const t = node._tooltip as Reactive<string> | ((state: Record<string, NodeState>) => string) | undefined
	if (t === undefined) return undefined
	const content = typeof t === 'function' ? t(props.state) : resolve(t)
	return content ? { content } : undefined
}

function clickButton(node: object): void {
	if (!hasCap(node, '_onClick')) return
	;(node._onClick as (state: Record<string, NodeState>, write: Writer) => void)?.(props.state, props.write)
}

function nodeKey(item: ChildNode, idx: number): string {
	return typeof item === 'object' && item !== null && hasIdCap(item) ? item.id : `n-${idx}`
}

function modelProp(item: object): string {
	return (item as RenderableValueNode)._modelProp
}

function updateEvent(item: object): string {
	return `update:${modelProp(item)}`
}

function updateValue(item: RenderableValueNode, v: unknown): void {
	const onChange = hasCap(item, '_onChange') ? (item._onChange as OnChangeFn | undefined) : undefined
	if (onChange) {
		const result = onChange(v as string, { override: (ov) => ({ __override: ov }) })
		if (result && typeof result === 'object' && '__override' in result) {
			writeNodeValue(item, props.state, props.write, result.__override as never)
			return
		}
	}
	writeNodeValue(item, props.state, props.write, v as never)
}
</script>

<template>
	<div :class="[flex ? 'flex flex-wrap gap-2' : 'space-y-4', 'w-full']">
		<template v-for="(item, idx) in nodes" :key="nodeKey(item, idx)">
			<template v-if="typeof item !== 'object' || item === null">
				<template v-if="typeof item === 'string'">{{ item }}</template>
				<component :is="item" v-else />
			</template>

			<template v-else-if="isShown(item)">
				<div
					:class="
						hasChildrenCap(item) && !hasValueCap(item)
							? 'w-full'
							: !getTitle(item)
								? 'contents'
								: undefined
					"
				>
					<div v-if="getTitle(item)" class="mb-2" :class="titleClass(titleDepth ?? 0)">
						<span
							v-html="renderString(getTitle(item)!).replace(/^<p>([\s\S]*)<\/p>\n?$/, '$1')"
						/><span v-if="needsAttention(item)" class="text-red">*</span>
					</div>

					<template v-if="hasChildrenCap(item) && !hasValueCap(item)">
						<NodeRenderer
							:nodes="resolveChildren(item, containerScope(item).state)"
							:state="containerScope(item).state"
							:write="containerScope(item).write"
							:on-image-upload="onImageUpload"
							:app-components="appComponents"
							:flex="(item as any)._layout !== 'column'"
							:title-depth="getTitle(item) !== undefined ? (titleDepth ?? 0) + 1 : titleDepth"
						/>
					</template>

					<template v-else-if="hasValueCap(item) && hasIdCap(item)">
						<component
							:is="resolveComponent(item as RenderableValueNode)"
							v-bind="componentProps(item as RenderableValueNode)"
							:[modelProp(item)]="getEffectiveValue(item as RenderableValueNode, state[item.id], state)"
							@[updateEvent(item)]="(v: unknown) => updateValue(item as RenderableValueNode, v)"
						/>
					</template>

					<template v-else-if="hasCap(item, '_onClick')">
						<ButtonStyled :circular="!!(item as any)._icon && !(item as any).label">
							<button
								v-tooltip="resolveTooltip(item)"
								:disabled="!isEnabled(item as any)"
								:aria-label="(item as any)._icon ? (item as any).label : undefined"
								@click="clickButton(item)"
							>
								<component :is="(item as any)._icon" v-if="(item as any)._icon" />
								<template v-else>{{ (item as any).label }}</template>
							</button>
						</ButtonStyled>
					</template>
				</div>
			</template>
		</template>

		<template v-for="(item, idx) in nodes" :key="`children-${nodeKey(item, idx)}`">
			<NodeRenderer
				v-if="
					typeof item === 'object' &&
					item !== null &&
					isShown(item) &&
					hasValueCap(item) &&
					hasIdCap(item) &&
					hasChildrenCap(item) &&
					isNodeActive(item, state[item.id], state) &&
					resolveChildren(item, valueScope(item).state).length
				"
				:nodes="resolveChildren(item, valueScope(item).state)"
				:state="valueScope(item).state"
				:write="valueScope(item).write"
				:on-image-upload="onImageUpload"
				:app-components="appComponents"
				:title-depth="getTitle(item) !== undefined ? (titleDepth ?? 0) + 1 : titleDepth"
				class="w-full"
			/>
		</template>
	</div>
</template>
