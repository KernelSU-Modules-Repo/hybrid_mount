/**
 * Copyright 2026 Hybrid Mount Developers
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import js from "@eslint/js";
import tseslint from "typescript-eslint";
import solid from "eslint-plugin-solid";
import globals from "globals";

export default tseslint.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["**/*.{ts,tsx}"],
    ...solid.configs["typescript-recommended"],
  },
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
  },
  {
    ignores: ["dist/**", "node_modules/**"],
  },
);
