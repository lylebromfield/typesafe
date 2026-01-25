/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        gruvbox: {
          bg: '#282828',       // Dark background
          bgHard: '#1d2021',   // Darker background (contrast)
          bgSoft: '#32302f',   // Lighter background
          fg: '#ebdbb2',       // Main text (off-white)
          fg0: '#fbf1c7',      // Bright text
          gray: '#928374',     // Gray
          red: '#cc241d',
          green: '#98971a',
          yellow: '#d79921',
          blue: '#458588',
          purple: '#b16286',
          aqua: '#689d6a',
          orange: '#d65d0e',   // Rusty orange (darker)
          orangeBright: '#fe8019', // Rusty orange (brighter)
        }
      },
      fontFamily: {
        sans: ['Inter', 'sans-serif'],
        mono: ['"JetBrains Mono"', 'monospace'],
        digital: ['"Anonymous Pro"', 'monospace'],
      },
      keyframes: {
        "fade-in-up": {
          "0%": { opacity: "0", transform: "translateY(20px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        }
      },
      animation: {
        "fade-in-up": "fade-in-up 0.8s ease-out forwards",
      }
    },
  },
  plugins: [],
}
