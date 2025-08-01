/**
 * @see https://prettier.io/docs/configuration
 * @type {import("prettier").Config}
 */
const config = {
  printWidth: 100,  
  semi: false,  
  singleQuote: true,  
  endOfLine: 'auto',
  plugins: ["prettier-plugin-tailwindcss"]
};

module.exports = config;