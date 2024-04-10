/** @type {import("prettier").Config} */
const config = {
  arrowParens: "always",
  printWidth: 120,
  quoteProps: "consistent",
  semi: false,
  singleQuote: true,
  trailingComma: "all",
  overrides: [
    {
      files: ".prettierrc",
      options: {
        parser: "json",
      },
    },
  ],
  plugins: [
    require.resolve("prettier-plugin-organize-imports"),
    require.resolve("prettier-plugin-tailwindcss"),
  ],
};

module.exports = config;
