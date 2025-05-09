declare module '*.svg?component' {
  import type { FunctionalComponent, SVGAttributes } from 'vue'

  const src: FunctionalComponent<SVGAttributes>
  export default src
}

declare module '*.webp' {
  const src: string
  export default src
}
