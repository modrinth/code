import plugin from 'tailwindcss/plugin'

const stops = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900] as const

export default plugin(function ({ addUtilities, theme }) {
  const rawColors = theme('colors') as Record<string, any>
  const leafColors: Record<string, string> = {}

  function flatten(obj: Record<string, any>, path: string[] = []) {
    for (const [key, val] of Object.entries(obj)) {
      if (typeof val === 'string') {
        leafColors[[...path, key].join('-')] = val
      } else if (val && typeof val === 'object') {
        flatten(val as Record<string, any>, [...path, key])
      }
    }
  }
  flatten(rawColors)

  const utils: Record<string, Record<string, string>> = {}
  for (const [name, colorValue] of Object.entries(leafColors)) {
    for (const stop of stops) {
      const ratio = stop < 500 ? 1 - stop / 1000 : (stop - 500) / 500
      const mixWith = stop < 500 ? 'white' : 'black'
      const pct = Math.round(ratio * 100)
      const cls = `${name}-${stop}`

      utils[`.bg-${cls}`] = {
        '--tw-bg-opacity': '1',
        'background-color': `color-mix(in srgb, ${colorValue} ${100 - pct}%, ${mixWith} ${pct}%)`,
      }
      utils[`.text-${cls}`] = {
        '--tw-text-opacity': '1',
        color: `color-mix(in srgb, ${colorValue} ${100 - pct}%, ${mixWith} ${pct}%)`,
      }
    }
  }

  // @ts-ignore
  addUtilities(utils, ['responsive', 'hover'])
})
