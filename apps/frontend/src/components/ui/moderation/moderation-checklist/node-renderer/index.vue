<script lang="ts" setup>
import { Button, IconButton } from '@modrinth/ui'
import { renderString } from '@modrinth/utils'

import type { NodeRendererProps, RenderableValueNode } from './types'
import { useNodeRenderer } from './use-node-renderer'

defineOptions({ name: 'NodeRenderer' })

const props = defineProps<NodeRendererProps>()

const {
	applyTweak,
	buttonIcon,
	buttonLabel,
	clickButton,
	childLayout,
	componentProps,
	containerScope,
	getEffectiveValue,
	getTitle,
	hasCap,
	hasChildrenCap,
	hasIdCap,
	hasValueCap,
	isEnabled,
	isNodeActive,
	isShown,
	modelProp,
	needsAttention,
	nodeKey,
	resolveChildren,
	resolveComponent,
	resolveTooltip,
	titleClass,
	tweakEnabled,
	tweakLabel,
	tweakTooltip,
	updateEvent,
	updateValue,
	valueScope,
	wrappedState,
} = useNodeRenderer(props)
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
						<!-- eslint-disable vue/no-v-html -- title text is author-controlled (stage definitions), not user input -->
						<span
							v-html="renderString(getTitle(item)!).replace(/^<p>([\s\S]*)<\/p>\n?$/, '$1')"
						/><span v-if="needsAttention(item)" class="text-red">*</span>
						<!-- eslint-enable vue/no-v-html -->
					</div>

					<template v-if="hasChildrenCap(item) && !hasValueCap(item)">
						<NodeRenderer
							:nodes="resolveChildren(item, containerScope(item).state)"
							:state="containerScope(item).state"
							:write="containerScope(item).write"
							:meta="meta"
							:on-image-upload="onImageUpload"
							:global-state="globalState"
							:flex="childLayout(item) !== 'column'"
							:title-depth="getTitle(item) !== undefined ? (titleDepth ?? 0) + 1 : titleDepth"
						/>
					</template>

					<template v-else-if="hasValueCap(item) && hasIdCap(item)">
						<component
							:is="resolveComponent(item as RenderableValueNode)"
							v-tooltip="resolveTooltip(item)"
							v-bind="componentProps(item as RenderableValueNode)"
							:[modelProp(item)]="
								getEffectiveValue(item as RenderableValueNode, state[item.id], wrappedState)
							"
							@[updateEvent(item)]="(value: unknown) => updateValue(item as RenderableValueNode, value)"
						/>
						<template
							v-for="(tweak, tweakIndex) in (item as RenderableValueNode)._tweaks ?? []"
							:key="`tweak-${tweakIndex}`"
						>
							<IconButton
								v-tooltip="tweakTooltip(tweak, item as RenderableValueNode)"
								:label="tweakLabel(tweak, item as RenderableValueNode)"
								:disabled="!tweakEnabled(tweak, item as RenderableValueNode)"
								@click="applyTweak(tweak, item as RenderableValueNode)"
							>
								<component :is="tweak.icon" aria-hidden="true" />
							</IconButton>
						</template>
					</template>

					<template v-else-if="hasCap(item, '_onClick') && buttonIcon(item)">
						<IconButton
							v-tooltip="resolveTooltip(item)"
							:disabled="!isEnabled(item)"
							:label="buttonLabel(item)"
							@click="clickButton(item)"
						>
							<component :is="buttonIcon(item)" aria-hidden="true" />
						</IconButton>
					</template>

					<template v-else-if="hasCap(item, '_onClick')">
						<Button
							v-tooltip="resolveTooltip(item)"
							:disabled="!isEnabled(item)"
							@click="clickButton(item)"
						>
							{{ buttonLabel(item) }}
						</Button>
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
					isNodeActive(item, state[item.id], wrappedState) &&
					resolveChildren(item, valueScope(item).state).length
				"
				:nodes="resolveChildren(item, valueScope(item).state)"
				:state="valueScope(item).state"
				:write="valueScope(item).write"
				:meta="meta"
				:on-image-upload="onImageUpload"
				:global-state="globalState"
				:title-depth="getTitle(item) !== undefined ? (titleDepth ?? 0) + 1 : titleDepth"
				class="w-full"
			/>
		</template>
	</div>
</template>
