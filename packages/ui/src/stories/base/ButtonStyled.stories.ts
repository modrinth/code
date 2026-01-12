import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ButtonStyled from '../../components/base/ButtonStyled.vue'

const meta = {
	title: 'Base/ButtonStyled',
	component: ButtonStyled,
	render: (args) => ({
		components: { ButtonStyled },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<ButtonStyled v-bind="args">
				<button @click="()=>{}">Button</button>
			</ButtonStyled>
		`,
	}),
} satisfies Meta<typeof ButtonStyled>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

// All colors showcase
export const AllColors: Story = {
	render: () => ({
		components: { ButtonStyled },
		template: /*html*/ `
			<div style="display: flex; flex-wrap: wrap; gap: 1rem; align-items: center;">
				<ButtonStyled color="standard"><button>Standard</button></ButtonStyled>
				<ButtonStyled color="brand"><button>Brand</button></ButtonStyled>
				<ButtonStyled color="red"><button>Red</button></ButtonStyled>
				<ButtonStyled color="orange"><button>Orange</button></ButtonStyled>
				<ButtonStyled color="green"><button>Green</button></ButtonStyled>
				<ButtonStyled color="blue"><button>Blue</button></ButtonStyled>
				<ButtonStyled color="purple"><button>Purple</button></ButtonStyled>
			</div>
		`,
	}),
}

// All sizes showcase
export const AllSizes: Story = {
	render: () => ({
		components: { ButtonStyled },
		template: /*html*/ `
			<div style="display: flex; flex-wrap: wrap; gap: 1rem; align-items: center;">
				<ButtonStyled size="small" color="standard"><button>Small</button></ButtonStyled>
				<ButtonStyled size="standard" color="standard"><button>Standard</button></ButtonStyled>
				<ButtonStyled size="large" color="standard"><button>Large</button></ButtonStyled>
			</div>
		`,
	}),
}

// All types showcase
export const AllTypes: Story = {
	render: () => ({
		components: { ButtonStyled },
		template: /*html*/ `
			<div style="display: flex; flex-wrap: wrap; gap: 1rem; align-items: center;">
				<ButtonStyled type="standard" color="standard"><button>Standard</button></ButtonStyled>
				<ButtonStyled type="outlined" color="standard"><button>Outlined</button></ButtonStyled>
				<ButtonStyled type="transparent" color="standard"><button>Transparent</button></ButtonStyled>
				<ButtonStyled type="highlight" color="standard"><button>Highlight</button></ButtonStyled>
				<ButtonStyled type="highlight-colored-text" color="standard"><button>Highlight Colored</button></ButtonStyled>
				<ButtonStyled type="chip" color="standard"><button>Chip</button></ButtonStyled>
				<ButtonStyled color="standard"><button disabled>Disabled</button></ButtonStyled>
			</div>
		`,
	}),
}
