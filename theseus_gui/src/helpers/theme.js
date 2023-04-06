/**
 * Sets the html tag in the DOM to have the dark-mode class
 */
export const setDarkTheme = () => {
  document.getElementsByTagName('html')[0].classList.remove('light-mode')
  document.getElementsByTagName('html')[0].classList.add('dark-mode')
}

/**
 * Sets the html tag in the DOM to have the light-mode class
 */
export const setLightTheme = () => {
  document.getElementsByTagName('html')[0].classList.remove('dark-mode')
  document.getElementsByTagName('html')[0].classList.add('light-mode')
}

// Put future DOM theme setting below
