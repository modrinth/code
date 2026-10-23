/**
 * The base themes highlight.js ships, by file name, each loaded only once picked. Generated from
 * `highlight.js/styles` (leaving out its `base16` folder of variants).
 */
export const HIGHLIGHT_JS_THEMES: { id: string; label: string; load: () => Promise<string> }[] = [
	{
		id: '1c-light',
		label: '1C Light',
		load: () => import('highlight.js/styles/1c-light.min.css?inline').then((css) => css.default),
	},
	{
		id: 'a11y-dark',
		label: 'A11y Dark',
		load: () => import('highlight.js/styles/a11y-dark.min.css?inline').then((css) => css.default),
	},
	{
		id: 'a11y-light',
		label: 'A11y Light',
		load: () => import('highlight.js/styles/a11y-light.min.css?inline').then((css) => css.default),
	},
	{
		id: 'agate',
		label: 'Agate',
		load: () => import('highlight.js/styles/agate.min.css?inline').then((css) => css.default),
	},
	{
		id: 'an-old-hope',
		label: 'An Old Hope',
		load: () => import('highlight.js/styles/an-old-hope.min.css?inline').then((css) => css.default),
	},
	{
		id: 'androidstudio',
		label: 'Android Studio',
		load: () =>
			import('highlight.js/styles/androidstudio.min.css?inline').then((css) => css.default),
	},
	{
		id: 'arduino-light',
		label: 'Arduino Light',
		load: () =>
			import('highlight.js/styles/arduino-light.min.css?inline').then((css) => css.default),
	},
	{
		id: 'arta',
		label: 'Arta',
		load: () => import('highlight.js/styles/arta.min.css?inline').then((css) => css.default),
	},
	{
		id: 'ascetic',
		label: 'Ascetic',
		load: () => import('highlight.js/styles/ascetic.min.css?inline').then((css) => css.default),
	},
	{
		id: 'atom-one-dark',
		label: 'Atom One Dark',
		load: () =>
			import('highlight.js/styles/atom-one-dark.min.css?inline').then((css) => css.default),
	},
	{
		id: 'atom-one-dark-reasonable',
		label: 'Atom One Dark Reasonable',
		load: () =>
			import('highlight.js/styles/atom-one-dark-reasonable.min.css?inline').then(
				(css) => css.default,
			),
	},
	{
		id: 'atom-one-light',
		label: 'Atom One Light',
		load: () =>
			import('highlight.js/styles/atom-one-light.min.css?inline').then((css) => css.default),
	},
	{
		id: 'brown-paper',
		label: 'Brown Paper',
		load: () => import('highlight.js/styles/brown-paper.min.css?inline').then((css) => css.default),
	},
	{
		id: 'codepen-embed',
		label: 'CodePen Embed',
		load: () =>
			import('highlight.js/styles/codepen-embed.min.css?inline').then((css) => css.default),
	},
	{
		id: 'color-brewer',
		label: 'Color Brewer',
		load: () =>
			import('highlight.js/styles/color-brewer.min.css?inline').then((css) => css.default),
	},
	{
		id: 'cybertopia-cherry',
		label: 'Cybertopia Cherry',
		load: () =>
			import('highlight.js/styles/cybertopia-cherry.min.css?inline').then((css) => css.default),
	},
	{
		id: 'cybertopia-dimmer',
		label: 'Cybertopia Dimmer',
		load: () =>
			import('highlight.js/styles/cybertopia-dimmer.min.css?inline').then((css) => css.default),
	},
	{
		id: 'cybertopia-icecap',
		label: 'Cybertopia Icecap',
		load: () =>
			import('highlight.js/styles/cybertopia-icecap.min.css?inline').then((css) => css.default),
	},
	{
		id: 'cybertopia-saturated',
		label: 'Cybertopia Saturated',
		load: () =>
			import('highlight.js/styles/cybertopia-saturated.min.css?inline').then((css) => css.default),
	},
	{
		id: 'dark',
		label: 'Dark',
		load: () => import('highlight.js/styles/dark.min.css?inline').then((css) => css.default),
	},
	{
		id: 'default',
		label: 'Default',
		load: () => import('highlight.js/styles/default.min.css?inline').then((css) => css.default),
	},
	{
		id: 'devibeans',
		label: 'Devibeans',
		load: () => import('highlight.js/styles/devibeans.min.css?inline').then((css) => css.default),
	},
	{
		id: 'docco',
		label: 'Docco',
		load: () => import('highlight.js/styles/docco.min.css?inline').then((css) => css.default),
	},
	{
		id: 'far',
		label: 'Far',
		load: () => import('highlight.js/styles/far.min.css?inline').then((css) => css.default),
	},
	{
		id: 'felipec',
		label: 'Felipec',
		load: () => import('highlight.js/styles/felipec.min.css?inline').then((css) => css.default),
	},
	{
		id: 'foundation',
		label: 'Foundation',
		load: () => import('highlight.js/styles/foundation.min.css?inline').then((css) => css.default),
	},
	{
		id: 'github',
		label: 'GitHub',
		load: () => import('highlight.js/styles/github.min.css?inline').then((css) => css.default),
	},
	{
		id: 'github-dark',
		label: 'GitHub Dark',
		load: () => import('highlight.js/styles/github-dark.min.css?inline').then((css) => css.default),
	},
	{
		id: 'github-dark-dimmed',
		label: 'GitHub Dark Dimmed',
		load: () =>
			import('highlight.js/styles/github-dark-dimmed.min.css?inline').then((css) => css.default),
	},
	{
		id: 'gml',
		label: 'GML',
		load: () => import('highlight.js/styles/gml.min.css?inline').then((css) => css.default),
	},
	{
		id: 'googlecode',
		label: 'Google Code',
		load: () => import('highlight.js/styles/googlecode.min.css?inline').then((css) => css.default),
	},
	{
		id: 'gradient-dark',
		label: 'Gradient Dark',
		load: () =>
			import('highlight.js/styles/gradient-dark.min.css?inline').then((css) => css.default),
	},
	{
		id: 'gradient-light',
		label: 'Gradient Light',
		load: () =>
			import('highlight.js/styles/gradient-light.min.css?inline').then((css) => css.default),
	},
	{
		id: 'grayscale',
		label: 'Grayscale',
		load: () => import('highlight.js/styles/grayscale.min.css?inline').then((css) => css.default),
	},
	{
		id: 'hybrid',
		label: 'Hybrid',
		load: () => import('highlight.js/styles/hybrid.min.css?inline').then((css) => css.default),
	},
	{
		id: 'idea',
		label: 'Idea',
		load: () => import('highlight.js/styles/idea.min.css?inline').then((css) => css.default),
	},
	{
		id: 'intellij-light',
		label: 'IntelliJ Light',
		load: () =>
			import('highlight.js/styles/intellij-light.min.css?inline').then((css) => css.default),
	},
	{
		id: 'ir-black',
		label: 'IR Black',
		load: () => import('highlight.js/styles/ir-black.min.css?inline').then((css) => css.default),
	},
	{
		id: 'isbl-editor-dark',
		label: 'ISBL Editor Dark',
		load: () =>
			import('highlight.js/styles/isbl-editor-dark.min.css?inline').then((css) => css.default),
	},
	{
		id: 'isbl-editor-light',
		label: 'ISBL Editor Light',
		load: () =>
			import('highlight.js/styles/isbl-editor-light.min.css?inline').then((css) => css.default),
	},
	{
		id: 'kimbie-dark',
		label: 'Kimbie Dark',
		load: () => import('highlight.js/styles/kimbie-dark.min.css?inline').then((css) => css.default),
	},
	{
		id: 'kimbie-light',
		label: 'Kimbie Light',
		load: () =>
			import('highlight.js/styles/kimbie-light.min.css?inline').then((css) => css.default),
	},
	{
		id: 'lightfair',
		label: 'Lightfair',
		load: () => import('highlight.js/styles/lightfair.min.css?inline').then((css) => css.default),
	},
	{
		id: 'lioshi',
		label: 'Lioshi',
		load: () => import('highlight.js/styles/lioshi.min.css?inline').then((css) => css.default),
	},
	{
		id: 'magula',
		label: 'Magula',
		load: () => import('highlight.js/styles/magula.min.css?inline').then((css) => css.default),
	},
	{
		id: 'mono-blue',
		label: 'Mono Blue',
		load: () => import('highlight.js/styles/mono-blue.min.css?inline').then((css) => css.default),
	},
	{
		id: 'monokai',
		label: 'Monokai',
		load: () => import('highlight.js/styles/monokai.min.css?inline').then((css) => css.default),
	},
	{
		id: 'monokai-sublime',
		label: 'Monokai Sublime',
		load: () =>
			import('highlight.js/styles/monokai-sublime.min.css?inline').then((css) => css.default),
	},
	{
		id: 'night-owl',
		label: 'Night Owl',
		load: () => import('highlight.js/styles/night-owl.min.css?inline').then((css) => css.default),
	},
	{
		id: 'nnfx-dark',
		label: 'NNFX Dark',
		load: () => import('highlight.js/styles/nnfx-dark.min.css?inline').then((css) => css.default),
	},
	{
		id: 'nnfx-light',
		label: 'NNFX Light',
		load: () => import('highlight.js/styles/nnfx-light.min.css?inline').then((css) => css.default),
	},
	{
		id: 'nord',
		label: 'Nord',
		load: () => import('highlight.js/styles/nord.min.css?inline').then((css) => css.default),
	},
	{
		id: 'obsidian',
		label: 'Obsidian',
		load: () => import('highlight.js/styles/obsidian.min.css?inline').then((css) => css.default),
	},
	{
		id: 'panda-syntax-dark',
		label: 'Panda Syntax Dark',
		load: () =>
			import('highlight.js/styles/panda-syntax-dark.min.css?inline').then((css) => css.default),
	},
	{
		id: 'panda-syntax-light',
		label: 'Panda Syntax Light',
		load: () =>
			import('highlight.js/styles/panda-syntax-light.min.css?inline').then((css) => css.default),
	},
	{
		id: 'paraiso-dark',
		label: 'Paraiso Dark',
		load: () =>
			import('highlight.js/styles/paraiso-dark.min.css?inline').then((css) => css.default),
	},
	{
		id: 'paraiso-light',
		label: 'Paraiso Light',
		load: () =>
			import('highlight.js/styles/paraiso-light.min.css?inline').then((css) => css.default),
	},
	{
		id: 'pojoaque',
		label: 'Pojoaque',
		load: () => import('highlight.js/styles/pojoaque.min.css?inline').then((css) => css.default),
	},
	{
		id: 'purebasic',
		label: 'PureBasic',
		load: () => import('highlight.js/styles/purebasic.min.css?inline').then((css) => css.default),
	},
	{
		id: 'qtcreator-dark',
		label: 'Qt Creator Dark',
		load: () =>
			import('highlight.js/styles/qtcreator-dark.min.css?inline').then((css) => css.default),
	},
	{
		id: 'qtcreator-light',
		label: 'Qt Creator Light',
		load: () =>
			import('highlight.js/styles/qtcreator-light.min.css?inline').then((css) => css.default),
	},
	{
		id: 'rainbow',
		label: 'Rainbow',
		load: () => import('highlight.js/styles/rainbow.min.css?inline').then((css) => css.default),
	},
	{
		id: 'rose-pine',
		label: 'Rose Pine',
		load: () => import('highlight.js/styles/rose-pine.min.css?inline').then((css) => css.default),
	},
	{
		id: 'rose-pine-dawn',
		label: 'Rose Pine Dawn',
		load: () =>
			import('highlight.js/styles/rose-pine-dawn.min.css?inline').then((css) => css.default),
	},
	{
		id: 'rose-pine-moon',
		label: 'Rose Pine Moon',
		load: () =>
			import('highlight.js/styles/rose-pine-moon.min.css?inline').then((css) => css.default),
	},
	{
		id: 'routeros',
		label: 'RouterOS',
		load: () => import('highlight.js/styles/routeros.min.css?inline').then((css) => css.default),
	},
	{
		id: 'school-book',
		label: 'School Book',
		load: () => import('highlight.js/styles/school-book.min.css?inline').then((css) => css.default),
	},
	{
		id: 'shades-of-purple',
		label: 'Shades Of Purple',
		load: () =>
			import('highlight.js/styles/shades-of-purple.min.css?inline').then((css) => css.default),
	},
	{
		id: 'srcery',
		label: 'Srcery',
		load: () => import('highlight.js/styles/srcery.min.css?inline').then((css) => css.default),
	},
	{
		id: 'stackoverflow-dark',
		label: 'Stack Overflow Dark',
		load: () =>
			import('highlight.js/styles/stackoverflow-dark.min.css?inline').then((css) => css.default),
	},
	{
		id: 'stackoverflow-light',
		label: 'Stack Overflow Light',
		load: () =>
			import('highlight.js/styles/stackoverflow-light.min.css?inline').then((css) => css.default),
	},
	{
		id: 'sunburst',
		label: 'Sunburst',
		load: () => import('highlight.js/styles/sunburst.min.css?inline').then((css) => css.default),
	},
	{
		id: 'tokyo-night-dark',
		label: 'Tokyo Night Dark',
		load: () =>
			import('highlight.js/styles/tokyo-night-dark.min.css?inline').then((css) => css.default),
	},
	{
		id: 'tokyo-night-light',
		label: 'Tokyo Night Light',
		load: () =>
			import('highlight.js/styles/tokyo-night-light.min.css?inline').then((css) => css.default),
	},
	{
		id: 'tomorrow-night-blue',
		label: 'Tomorrow Night Blue',
		load: () =>
			import('highlight.js/styles/tomorrow-night-blue.min.css?inline').then((css) => css.default),
	},
	{
		id: 'tomorrow-night-bright',
		label: 'Tomorrow Night Bright',
		load: () =>
			import('highlight.js/styles/tomorrow-night-bright.min.css?inline').then((css) => css.default),
	},
	{
		id: 'vs',
		label: 'VS',
		load: () => import('highlight.js/styles/vs.min.css?inline').then((css) => css.default),
	},
	{
		id: 'vs2015',
		label: 'VS 2015',
		load: () => import('highlight.js/styles/vs2015.min.css?inline').then((css) => css.default),
	},
	{
		id: 'xcode',
		label: 'Xcode',
		load: () => import('highlight.js/styles/xcode.min.css?inline').then((css) => css.default),
	},
	{
		id: 'xt256',
		label: 'XT256',
		load: () => import('highlight.js/styles/xt256.min.css?inline').then((css) => css.default),
	},
]
