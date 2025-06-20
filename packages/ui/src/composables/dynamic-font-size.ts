import { computed, onMounted, onUnmounted, type Ref } from 'vue'
import { useElementSize } from '@vueuse/core'

export interface DynamicFontSizeOptions {
  containerElement: Ref<HTMLElement | null>
  text: Ref<string | undefined>
  baseFontSize?: number
  minFontSize?: number
  maxFontSize?: number
  availableWidthRatio?: number
  maxContainerWidth?: number
  padding?: number
  fontFamily?: string
  fontWeight?: string | number
}

export function useDynamicFontSize(options: DynamicFontSizeOptions) {
  const {
    containerElement,
    text,
    baseFontSize = 1.25,
    minFontSize = 0.75,
    maxFontSize = 2,
    availableWidthRatio = 0.9,
    maxContainerWidth = 400,
    padding = 24,
    fontFamily = 'inherit',
    fontWeight = 'inherit',
  } = options

  const { width: containerWidth } = useElementSize(containerElement)
  let measurementElement: HTMLElement | null = null

  const createMeasurementElement = () => {
    if (measurementElement) return measurementElement

    measurementElement = document.createElement('div')
    measurementElement.style.cssText = `
      position: absolute;
      top: -9999px;
      left: -9999px;
      opacity: 0;
      pointer-events: none;
      white-space: nowrap;
      font-family: ${fontFamily};
      font-weight: ${fontWeight};
    `
    measurementElement.setAttribute('aria-hidden', 'true')
    document.body.appendChild(measurementElement)

    return measurementElement
  }

  const cleanupMeasurementElement = () => {
    if (measurementElement?.parentNode) {
      measurementElement.parentNode.removeChild(measurementElement)
      measurementElement = null
    }
  }

  const measureTextWidth = (textContent: string, fontSize: number): number => {
    if (!textContent) return 0

    const element = createMeasurementElement()
    element.style.fontSize = `${fontSize}rem`
    element.textContent = textContent

    return element.getBoundingClientRect().width
  }

  const findOptimalFontSize = (textContent: string, availableWidth: number): number => {
    let low = minFontSize
    let high = maxFontSize
    let bestSize = minFontSize

    const maxWidth = measureTextWidth(textContent, maxFontSize)
    if (maxWidth <= availableWidth) return maxFontSize

    for (let i = 0; i < 8; i++) {
      const mid = (low + high) / 2
      const width = measureTextWidth(textContent, mid)

      if (width <= availableWidth) {
        bestSize = mid
        low = mid
      } else {
        high = mid
      }

      if (high - low < 0.01) break
    }

    return Math.max(bestSize, minFontSize)
  }

  const fontSize = computed(() => {
    if (!text.value || !containerWidth.value) return `${baseFontSize}rem`

    const availableWidth =
      Math.min(containerWidth.value * availableWidthRatio, maxContainerWidth) - padding

    const baseWidth = measureTextWidth(text.value, baseFontSize)
    if (baseWidth <= availableWidth) return `${baseFontSize}rem`

    const optimalSize = findOptimalFontSize(text.value, availableWidth)
    return `${optimalSize}rem`
  })

  onMounted(createMeasurementElement)
  onUnmounted(cleanupMeasurementElement)

  return {
    fontSize,
    containerWidth,
    cleanup: cleanupMeasurementElement,
  }
}
