import { defineConfig, globalIgnores } from "eslint/config";
import nextVitals from "eslint-config-next/core-web-vitals";
import nextTypescript from "eslint-config-next/typescript";

const mahfelRules = {
  rules: {
    "no-comments": {
      meta: {
        type: "suggestion",
        docs: {
          description: "Disallow source comments except ESLint disable directives",
        },
        schema: [],
        messages: {
          forbidden: "Source comments are not allowed; write self-explanatory code instead.",
        },
      },
      create(context) {
        return {
          Program() {
            for (const comment of context.sourceCode.getAllComments()) {
              const value = comment.value.trim();
              const isEslintDisable = /^eslint-disable(?:-next-line|-line)?(?:\s|$)/u.test(value);

              if (!isEslintDisable) {
                context.report({
                  loc: comment.loc,
                  messageId: "forbidden",
                });
              }
            }
          },
        };
      },
    },
  },
};

export default defineConfig([
  ...nextVitals,
  ...nextTypescript,
  {
    files: ["**/*.{js,jsx,mjs,cjs,ts,tsx}"],
    plugins: {
      mahfel: mahfelRules,
    },
    linterOptions: {
      reportUnusedDisableDirectives: "error",
    },
    rules: {
      "array-callback-return": "error",
      "block-scoped-var": "error",
      "consistent-return": "error",
      curly: ["error", "all"],
      eqeqeq: ["error", "always", { null: "ignore" }],
      "mahfel/no-comments": "error",
      "no-alert": "error",
      "no-console": "error",
      "no-debugger": "error",
      "no-duplicate-imports": "error",
      "no-else-return": "error",
      "no-eval": "error",
      "no-implied-eval": "error",
      "no-lone-blocks": "error",
      "no-multi-assign": "error",
      "no-nested-ternary": "error",
      "no-new-wrappers": "error",
      "no-param-reassign": "error",
      "no-return-assign": "error",
      "no-sequences": "error",
      "no-useless-concat": "error",
      "no-useless-return": "error",
      "no-var": "error",
      "object-shorthand": "error",
      "prefer-const": "error",
      "prefer-template": "error",
      radix: "error",
      yoda: "error",
      "@typescript-eslint/consistent-type-imports": [
        "error",
        {
          disallowTypeAnnotations: false,
          fixStyle: "inline-type-imports",
          prefer: "type-imports",
        },
      ],
      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/no-non-null-assertion": "error",
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
        },
      ],
      "react/jsx-no-useless-fragment": "error",
      "react/self-closing-comp": "error",
    },
  },
  globalIgnores([
    ".next/**",
    "out/**",
    "build/**",
    "storybook-static/**",
    "lib/api/generated/**",
    "lib/websocket/generated/**",
    "next-env.d.ts",
  ]),
]);
