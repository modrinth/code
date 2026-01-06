# Storybook Story Creation Guide

This document provides instructions for AI assistants when creating Storybook stories for Vue components in the `@modrinth/ui` package.

## File Location

Stories should be placed in a `stories` subdirectory next to the component's directory:

- Component: `src/components/base/MyComponent.vue`
- Story: `src/stories/base/MyComponent.stories.ts`

Example with modal components:

- Component: `src/components/modal/MyModal.vue`
- Story: `src/stories/modal/MyModal.stories.ts`

## Basic Story Structure

```typescript
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import MyComponent from '../../components/base/MyComponent.vue'

const meta = {
	title: 'Base/MyComponent', // Category/ComponentName
	component: MyComponent,
} satisfies Meta<typeof MyComponent>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		// Default prop values
	},
}
```

## Key Principles

### 1. Let Storybook Auto-Infer Props

**DO NOT** manually define `argTypes`.

```typescript
// ❌ BAD - Don't include prop types
const meta = {
	argTypes: {
		size: { control: 'select', options: ['small', 'medium', 'large'] },
	},
}

// ✅ GOOD - Let Storybook infer from component
const meta = {
	component: MyComponent,
}
```

### 2. Use Render Functions for Slot Content

When a component uses slots, provide a render function:

```typescript
const meta = {
	component: Accordion,
	render: (args) => ({
		components: { Accordion },
		setup() {
			return { args }
		},
		template: /* html */ `
            <Accordion v-bind="args">
                <template #title>Click to expand</template>
                <p>Accordion content here.</p>
            </Accordion>
        `,
	}),
} satisfies Meta<typeof Accordion>
```

### 3. Keep Stories Concise

Instead of creating individual stories for every prop variant, use showcase stories:

Make sure to type it as `StoryObj` when using render functions.

```typescript
// ❌ BAD - Too many individual stories
export const Small: Story = { args: { size: 'small' } }
export const Medium: Story = { args: { size: 'medium' } }
export const Large: Story = { args: { size: 'large' } }

// ✅ GOOD - One showcase story
export const AllSizes: StoryObj = {
	render: () => ({
		components: { MyComponent },
		template: /* html */ `
            <div class="flex gap-4">
                <MyComponent size="small">Small</MyComponent>
                <MyComponent size="medium">Medium</MyComponent>
                <MyComponent size="large">Large</MyComponent>
            </div>
        `,
	}),
}
```

### 4. Required Stories

Each component should have:

- `Default` - Basic usage with controls
- `All[Variants]` - Showcase stories for major prop variations (e.g., `AllColors`, `AllSizes`, `AllTypes`)

### 5. Handling Generic Vue Components

For components with generics like `MyComponent<T>`, add // @ts-ignore

```typescript
const meta = {
	title: 'Base/Combobox',
	// @ts-ignore
	component: Combobox,
} satisfies Meta<typeof Combobox>
```

## Common Patterns

### Components with Icons

Import icons from `@modrinth/assets`:

```typescript
import { SearchIcon, ChevronDownIcon } from '@modrinth/assets'

export const WithIcon: Story = {
	render: () => ({
		components: { MyComponent, SearchIcon },
		template: /* html */ `
            <MyComponent>
                <SearchIcon />
                Search
            </MyComponent>
        `,
	}),
	args: {},
}
```

### Interactive Components (Modals, Dropdowns)

For components that need user interaction to show:

```typescript
export const Default: Story = {
	render: () => ({
		components: { Modal, ButtonStyled },
		setup() {
			const modalRef = ref<InstanceType<typeof Modal> | null>(null)
			return { modalRef }
		},
		template: /* html */ `
            <div>
                <ButtonStyled @click="modalRef?.show()">
                    <button>Open Modal</button>
                </ButtonStyled>
                <Modal ref="modalRef" header="Example Modal">
                    <p>Modal content</p>
                </Modal>
            </div>
        `,
	}),
	args: {},
}
```

### Components with v-model

```typescript
export const Default: Story = {
	render: () => ({
		components: { Toggle },
		setup() {
			const value = ref(false)
			return { value }
		},
		template: /* html */ `
            <Toggle v-model="value" />
        `,
	}),
	args: {},
}
```

## Things to Avoid

### 1. Don't Import from `@modrinth/ui` in Components

Components should use relative imports, not the package alias:

```typescript
// ❌ BAD - Causes circular dependency in Storybook
import { ButtonStyled } from '@modrinth/ui'

// ✅ GOOD - Use relative imports
import ButtonStyled from '../base/ButtonStyled.vue'
```

### 2. Object/Array Prop Defaults Must Be Factory Functions

This is a Vue requirement that `vue-docgen-plugin` enforces:

```typescript
// ❌ BAD - Will cause Storybook build error
defineProps<{ icon: Component }>()
withDefaults(defineProps<{ icon: Component }>(), {
	icon: TrashIcon,
})

// ✅ GOOD - Use factory function
withDefaults(defineProps<{ icon: Component }>(), {
	icon: () => TrashIcon,
})
```

## Dependencies Available in Storybook

The following are configured in `.storybook/preview.ts`:

- **VIntl**: `useVIntl()` and `formatMessage()` work automatically
- **Teleports**: `<Teleport to="#teleports">` has a target element
- **Tailwind CSS**: All Tailwind classes are available
- **Dark Mode**: Use `@storybook/addon-themes` for theme switching

## Example: Complete Story File

```typescript
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'
import Badge from '../../components/base/Badge.vue'

const meta = {
	title: 'Base/Badge',
	component: Badge,
	render: (args) => ({
		components: { Badge },
		setup() {
			return { args }
		},
		template: /* html */ `
            <Badge v-bind="args">Badge Text</Badge>
        `,
	}),
} satisfies Meta<typeof Badge>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		color: 'green',
	},
}

export const AllColors: StoryObj = {
	render: () => ({
		components: { Badge },
		template: /* html */ `
            <div class="flex flex-wrap gap-2">
                <Badge color="green">Green</Badge>
                <Badge color="red">Red</Badge>
                <Badge color="orange">Orange</Badge>
                <Badge color="blue">Blue</Badge>
                <Badge color="purple">Purple</Badge>
                <Badge color="gray">Gray</Badge>
            </div>
        `,
	}),
}

export const AllTypes: StoryObj = {
	render: () => ({
		components: { Badge },
		template: /* html */ `
            <div class="flex flex-wrap gap-2">
                <Badge type="default">Default</Badge>
                <Badge type="outlined">Outlined</Badge>
                <Badge type="highlight">Highlight</Badge>
            </div>
        `,
	}),
}
```
