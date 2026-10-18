import { defineComponent, type PropType, renderSlot, type Slots } from 'vue'

/**
 * Renders a slot handed over from another component (e.g. the workbench's panel slots, shown from
 * within dockview's panels). Compiled slots have to go through `renderSlot` to keep their block
 * tree intact; rendering one directly as a component (`<component :is="slot" />`) lets updates
 * patch the wrong nodes once its content has conditional parts.
 */
export const SlotOutlet = defineComponent({
	name: 'SlotOutlet',
	props: {
		slots: { type: Object as PropType<Slots>, required: true },
		name: { type: String, required: true },
	},
	setup(props) {
		return () => renderSlot(props.slots, props.name)
	},
})
