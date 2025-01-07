// Converts sRGB component to linear color space
function srgbToLinear(value: number) {
  if (value <= 0.04045) {
    return value / 12.92
  } else {
    return Math.pow((value + 0.055) / 1.055, 2.4)
  }
}

// https://en.wikipedia.org/wiki/Oklab_color_space#Conversions_between_color_spaces
export function rgbToOklchHue(rgb: number): number {
  // split out components and convert to linear color space
  const rLinear = srgbToLinear((rgb >> 16) & 0xff)
  const gLinear = srgbToLinear((rgb >> 8) & 0xff)
  const bLinear = srgbToLinear(rgb & 0xff)

  // convert linear RGB to CIE XYZ
  const l = 0.4122214708 * rLinear + 0.5363325363 * gLinear + 0.0514459929 * bLinear
  const m = 0.2119034982 * rLinear + 0.6806995451 * gLinear + 0.1073969566 * bLinear
  const s = 0.0883024619 * rLinear + 0.2817188376 * gLinear + 0.6299787005 * bLinear

  // convert CIE XYZ to LMS
  const l_ = Math.cbrt(l)
  const m_ = Math.cbrt(m)
  const s_ = Math.cbrt(s)

  // convert LMS to oklab
  const oklabA = 1.9779984951 * l_ + -2.428592205 * m_ + 0.4505937099 * s_
  const oklabB = 0.0259040371 * l_ + 0.7827717662 * m_ + -0.80867576 * s_

  // Convert cartesian oklab a and b to polar oklch hue
  const hRad = Math.atan2(oklabB, oklabA)
  const hDeg = (hRad * 180) / Math.PI

  return hDeg < 0 ? hDeg + 360 : hDeg
}
