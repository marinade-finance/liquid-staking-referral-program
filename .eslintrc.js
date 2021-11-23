module.exports = {
  env: {
    browser: false,
    es2021: true,
    mocha: true,
    node: true,
  },
  extends: [
    "standard",
    "plugin:prettier/recommended",
    "plugin:node/recommended",
    "plugin:@typescript-eslint/recommended",
  ],
  parser: "@typescript-eslint/parser",
  parserOptions: {
    ecmaVersion: 12,
    sourceType: "module",
  },
  plugins: ["@typescript-eslint"],
  rules: {},
  overrides: [
    {
      files: ["tests/**"],
      rules: {
        camelcase: "off",
        "node/no-missing-import": "off",
      },
    },
    {
      files: ["migrations/**", "tests/**"],
      rules: {
        "node/no-unpublished-import": "off",
        "node/no-unsupported-features/es-syntax": "off",
      },
    },
  ],
};
