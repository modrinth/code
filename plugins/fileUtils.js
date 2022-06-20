import { formatBytes } from '~/plugins/shorthands'

/**
 * @param {File | Blob} file the file to validate
 * @param {{ maxSize: number, alertOnInvalid: boolean }} validationOptions the
 * constraints to validate the file against
 * @param validationOptions.maxSize the max file size in bytes
 * @param validationOptions.alertOnInvalid if an alert should pop up describing
 * each validation error
 * @returns `true` if the file is valid; `false` otherise
 */
export const fileIsValid = (file, validationOptions) => {
  const { maxSize, alertOnInvalid } = validationOptions
  if (maxSize !== null && maxSize !== undefined && file.size > maxSize) {
    console.log(`File size: ${file.size}, max size: ${maxSize}`)
    if (alertOnInvalid) {
      alert(
        `File ${file.name} is too big! Must be less than ${formatBytes(
          maxSize
        )}`
      )
    }
    return false
  }

  return true
}
