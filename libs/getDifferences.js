const equalArrays = (arr1, arr2) =>
  arr1.length === arr2.length &&
  arr1.every((element, index) => element === arr2[index])

const isObject = (obj) => obj instanceof Object && !Array.isArray(obj)

const isArray = (arr) => Array.isArray(arr)

export default function getDifferences(obj1, obj2) {
  const obj3 = {}
  for (const key of Object.keys(obj1)) {
    const val1 = obj1[key]
    const val2 = obj2[key]
    const areArrays = isArray(val1) && isArray(val2)
    const areObjects = isObject(val1) && isObject(val2)
    if (areObjects) {
      const diff = getDifferences(val1, val2)
      if (diff) obj3[key] = diff
    } else if (areArrays && !equalArrays(val1, val2)) obj3[key] = val2
    else if (val1 !== val2) obj3[key] = val2
  }
  return !!Object.keys(obj3).length && obj3
}
