/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./pages/**/*.{ts,tsx}",
    "./components/**/*.{ts,tsx}"
  ],
  theme: {
    extend: {
      animation: {
        flash: "flash 1.1s ease-in-out infinite",
      },
      keyframes: {
        flash: {
          "0%, 100%": {
            opacity: 1,
          },
          "50%": {
            opacity: 0,
          }
        }
      }
    },
  },
  plugins: [],
}
