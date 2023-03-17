/**
 * Sets the dark-mode or light-mode class on <html> if dark mode is on.
 * @param {Boolean} isDarkMode Bool value indicating if dark mode is on.
 */
export const toggleTheme = (isDarkMode) => {
  if (isDarkMode) {
    document.getElementsByTagName('html')[0].classList.remove('light-mode')
    document.getElementsByTagName('html')[0].classList.add('dark-mode')
  } else {
    document.getElementsByTagName('html')[0].classList.remove('dark-mode')
    document.getElementsByTagName('html')[0].classList.add('light-mode')
  }
}
