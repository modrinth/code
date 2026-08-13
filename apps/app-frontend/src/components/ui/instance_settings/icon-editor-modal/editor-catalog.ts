import { defineMessages } from '@modrinth/ui'

import dirtBlock from '@/assets/instance-icons/dirt.png'
import duskBlock from '@/assets/instance-icons/dusk-block.png'
import duskBlockGold from '@/assets/instance-icons/dusk-block-gold-test.png'
import duskBlockRose from '@/assets/instance-icons/dusk-block-rose-test.png'
import duskBlockTeal from '@/assets/instance-icons/dusk-block-teal-test.png'

const names = defineMessages({
	yellow: {
		id: 'instance.icon-editor.background.yellow',
		defaultMessage: 'Yellow',
	},
	green: {
		id: 'instance.icon-editor.background.green',
		defaultMessage: 'Green',
	},
	purple: {
		id: 'instance.icon-editor.background.purple',
		defaultMessage: 'Purple',
	},
	blue: {
		id: 'instance.icon-editor.background.blue',
		defaultMessage: 'Blue',
	},
	orange: {
		id: 'instance.icon-editor.background.orange',
		defaultMessage: 'Orange',
	},
	red: {
		id: 'instance.icon-editor.background.red',
		defaultMessage: 'Red',
	},
	rose: {
		id: 'instance.icon-editor.background.rose',
		defaultMessage: 'Rose',
	},
	pink: {
		id: 'instance.icon-editor.background.pink',
		defaultMessage: 'Pink',
	},
	indigo: {
		id: 'instance.icon-editor.background.indigo',
		defaultMessage: 'Indigo',
	},
	lightGray: {
		id: 'instance.icon-editor.background.light-gray',
		defaultMessage: 'Light gray',
	},
	gray: {
		id: 'instance.icon-editor.background.gray',
		defaultMessage: 'Gray',
	},
	darkGray: {
		id: 'instance.icon-editor.background.dark-gray',
		defaultMessage: 'Dark gray',
	},
	dirtBlock: {
		id: 'instance.icon-editor.symbol.dirt-block',
		defaultMessage: 'Dirt block',
	},
	duskBlock: {
		id: 'instance.icon-editor.symbol.dusk-block',
		defaultMessage: 'Dusk block',
	},
	duskBlockRose: {
		id: 'instance.icon-editor.symbol.dusk-block-rose',
		defaultMessage: 'Test block',
	},
	duskBlockGold: {
		id: 'instance.icon-editor.symbol.dusk-block-gold',
		defaultMessage: 'Test block',
	},
	duskBlockTeal: {
		id: 'instance.icon-editor.symbol.dusk-block-teal',
		defaultMessage: 'Test block',
	},
})

export const backgroundOptions = [
	{ background: { type: 'color', value: '#fdd844' }, name: names.yellow },
	{ background: { type: 'color', value: '#ffa347' }, name: names.orange },
	{ background: { type: 'color', value: '#ff496e' }, name: names.rose },
	{ background: { type: 'color', value: '#f64447' }, name: names.red },
	{ background: { type: 'color', value: '#f468e6' }, name: names.pink },
	{ background: { type: 'color', value: '#c78aff' }, name: names.purple },
	{ background: { type: 'color', value: '#4f9cff' }, name: names.blue },
	{ background: { type: 'color', value: '#5a4eff' }, name: names.indigo },
	{ background: { type: 'color', value: '#1bd96a' }, name: names.green },
	{ background: { type: 'color', value: '#d9d9d9' }, name: names.lightGray },
	{ background: { type: 'color', value: '#4c4f58' }, name: names.gray },
	{ background: { type: 'color', value: '#252731' }, name: names.darkGray },
] as const

export const symbolOptions = [
	{ id: 'dirt_block', name: names.dirtBlock, asset: dirtBlock },
	{ id: 'dusk_block', name: names.duskBlock, asset: duskBlock },
	{ id: 'dusk_block_rose_test', name: names.duskBlockRose, asset: duskBlockRose },
	{ id: 'dusk_block_gold_test', name: names.duskBlockGold, asset: duskBlockGold },
	{ id: 'dusk_block_teal_test', name: names.duskBlockTeal, asset: duskBlockTeal },
] as const

export type BackgroundColor = (typeof backgroundOptions)[number]['background']['value']
export type SymbolId = (typeof symbolOptions)[number]['id']

export const DEFAULT_BACKGROUND_COLOR = '#c78aff' satisfies BackgroundColor
export const DEFAULT_SYMBOL_ID = 'dusk_block' satisfies SymbolId
