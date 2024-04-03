import type { Config } from 'tailwindcss'
import colors from 'tailwindcss/colors'


const config = {
  darkMode: 'selector',
  content: [
    '../../packages/ui/src/**/*.{ts,tsx}',
    './src/**/*.{ts,tsx}',
  ],
  theme: {
    extend: {
      colors: {
        transparent: 'transparent',
        current: 'currentColor',
        accent: {
          DEFAULT: 'hsl(var(--color-accent) / <alpha-value>)',
        },
        error: {
          DEFAULT: colors.red[500],
        },
        ink: {
          DEFAULT: 'hsl(var(--color-ink) / <alpha-value>)',
        },
        app: {
          DEFAULT: 'hsl(var(--color-app) / <alpha-value>)',
          box: {
            DEFAULT: 'hsl(var(--color-app-box) / <alpha-value>)',
            // dark: 'hsl(var(--color-app-box-dark) / <alpha-value>)',
          },
          line: 'hsl(var(--color-app-line) / <alpha-value>)',
          hover: 'hsl(var(--color-app-hover) / <alpha-value>)',
          overlay: 'hsl(var(--color-app-overlay) / <alpha-value>)',
        },
        toolbar: {
          DEFAULT: 'hsl(var(--color-toolbar) / <alpha-value>)',
          hover: 'hsl(var(--color-toolbar-hover) / <alpha-value>)',
          line: 'hsl(var(--color-toolbar-line) / <alpha-value>)',
        },
        sidebar: {
          DEFAULT: 'hsl(var(--color-sidebar) / <alpha-value>)',
          box: 'hsl(var(--color-sidebar-box) / <alpha-value>)',
          hover: 'hsl(var(--color-sidebar-hover) / <alpha-value>)',
          line: 'hsl(var(--color-sidebar-line) / <alpha-value>)',
        },
        // border: 'hsl(var(--border))',
        // input: 'hsl(var(--input))',
        // ring: 'hsl(var(--ring))',
        // background: 'hsl(var(--background))',
        // foreground: 'hsl(var(--foreground))',
        // primary: {
        //   DEFAULT: 'hsl(var(--primary))',
        //   foreground: 'hsl(var(--primary-foreground))',
        // },
        // secondary: {
        //   DEFAULT: 'hsl(var(--secondary))',
        //   foreground: 'hsl(var(--secondary-foreground))',
        // },
        // destructive: {
        //   DEFAULT: 'hsl(var(--destructive))',
        //   foreground: 'hsl(var(--destructive-foreground))',
        // },
        // muted: {
        //   DEFAULT: 'hsl(var(--muted))',
        //   foreground: 'hsl(var(--muted-foreground))',
        // },
        // accent: {
        //   DEFAULT: 'hsl(var(--accent))',
        //   foreground: 'hsl(var(--accent-foreground))',
        // },
        // popover: {
        //   DEFAULT: 'hsl(var(--popover))',
        //   foreground: 'hsl(var(--popover-foreground))',
        // },
        // card: {
        //   DEFAULT: 'hsl(var(--card))',
        //   foreground: 'hsl(var(--card-foreground))',
        // },
      },
      // 自定义的阴影值
      // boxShadow: {
      //   xs: '0 0.5px 1px 0.5px rgba(0, 0, 0, 0.1)',
      // },
    },
  },
  plugins: [require('tailwindcss-animate')],
} satisfies Config

export default config
