export const toggleTheme = (isDarkMode) => {
  if (isDarkMode) document.getElementsByTagName('html')[0].classList.add('dark-mode')
  else document.getElementsByTagName('html')[0].classList.remove('dark-mode')
}
