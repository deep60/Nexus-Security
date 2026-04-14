import js from "@eslint/js";
import tseslint from "typescript-eslint";
import globals from "globals";

export default tseslint.config(
  // Global ignores
  {
    ignores: [
      "artifacts/",
      "cache/",
      "coverage/",
      "deployments/",
      "node_modules/",
      "typechain-types/",
      "abis/",
    ],
  },

  // TypeScript files: scripts + tests + config
  {
    files: ["**/*.ts"],
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    languageOptions: {
      ecmaVersion: 2020,
      globals: {
        ...globals.node,
        ...globals.mocha,
      },
    },
    rules: {
      // Allow underscore-prefixed unused vars (convention for intentional skips)
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
          destructuredArrayIgnorePattern: "^_",
        },
      ],

      // Hardhat scripts use require() for dynamic imports
      "@typescript-eslint/no-require-imports": "off",

      // Allow empty catch blocks (common in deploy scripts for "dir exists" checks)
      "no-empty": ["error", { allowEmptyCatch: true }],
    },
  },

  // Test files: allow Chai assertion expressions (expect().to.be.true)
  {
    files: ["test/**/*.ts"],
    rules: {
      "@typescript-eslint/no-unused-expressions": "off",
    },
  }
);
