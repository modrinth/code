/**
 * @see https://prettier.io/docs/configuration
 * @type {import("prettier").Config}
 */
const config = {
  semi: false,  
  singleQuote: true,  
  plugins: ["prettier-plugin-tailwindcss"]
};

module.exports = config;