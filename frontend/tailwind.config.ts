// Tailwind CSS configuration for personal-ledger-react
// This file configures Tailwind CSS for the React app, specifying content paths for style purging,
// theme extensions for custom design tokens, and plugins for additional utilities.
// It ensures efficient CSS generation and supports shadcn/ui components.

import { Config } from "tailwindcss";

export default {
  // Paths to files containing Tailwind classes for purging unused styles
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  // Theme configuration for customizing design tokens
  theme: {
    // Extend the default theme with custom colors, fonts, etc.
    extend: {},
  },
  // Array of Tailwind plugins for additional utilities and components
  plugins: [],
} satisfies Config;
