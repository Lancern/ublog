/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./pages/**/*.{ts,tsx}", "./components/**/*.{ts,tsx}"],
  theme: {
    fontFamily: {
      sans: '-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,"Helvetica Neue",Arial,"Microsoft Yahei",sans-serif,"Apple Color Emoji","Segoe UI Emoji","Segoe UI Symbol"',
    },
    extend: {
      animation: {
        flash: "flash 1.1s ease-in-out infinite",
        loading1: "loading1 1.1s ease-in-out infinite",
        loading2: "loading2 1.1s ease-in-out infinite",
        menuFadeIn: "menuFadeIn 0.2s ease-in-out",
        menuFadeOut: "menuFadeOut 0.2s ease-in-out",
      },
      keyframes: {
        flash: {
          "0%, 100%": {
            opacity: 1,
          },
          "50%": {
            opacity: 0,
          },
        },
        loading1: {
          "0%, 100%": {
            transform: "translateY(-200%)",
            animationTimingFunction: "cubic-bezier(0.8,0,1,1)",
          },
          "50%": {
            transform: "none",
            animationTimingFunction: "cubic-bezier(0,0,0.2,1)",
          },
        },
        loading2: {
          "0%, 100%": {
            transform: "none",
            animationTimingFunction: "cubic-bezier(0,0,0.2,1)",
          },
          "50%": {
            transform: "translateY(-200%)",
            animationTimingFunction: "cubic-bezier(0.8,0,1,1)",
          },
        },
        menuFadeIn: {
          "0%": {
            opacity: 0,
            transform: "translateY(-0.5rem)",
          },
          "100%": {
            opacity: 1,
          },
        },
        menuFadeOut: {
          "0%": {
            opacity: 1,
          },
          "100%": {
            opacity: 0,
            transform: "translateY(-0.5rem)",
          },
        },
      },
    },
  },
  safelist: [
    "font-black",
    "italic",
    "line-through",
    "underline",
    "pl-2",
    "pl-4",
    "pl-6",
    "border-l-2",
    "border-l-black",
    "text-black dark",
    "text-white",
    "text-gray-400",
    "text-amber-800",
    "text-orange-600",
    "text-yellow-400",
    "text-green-600",
    "text-blue-600",
    "text-purple-600",
    "text-pink-600",
    "text-red-600",
    "bg-gray-200",
    "bg-amber-700",
    "bg-orange-200",
    "bg-yellow-200",
    "bg-green-200",
    "bg-blue-200",
    "bg-purple-200",
    "bg-pink-200",
    "bg-red-200",
    "animate-fadeIn",
    "animate-fadeOut",
    "dark:text-gray-600",
    "dark:text-amber-700",
    "dark:text-orange-400",
    "dark:text-yellow-600",
    "dark:text-green-400",
    "dark:text-blue-400",
    "dark:text-purple-400",
    "dark:text-pink-400",
    "dark:text-red-400",
    "dark:bg-gray-700",
    "dark:bg-orange-700",
    "dark:bg-yellow-700",
    "dark:bg-green-700",
    "dark:bg-blue-700",
    "dark:bg-purple-700",
    "dark:bg-pink-700",
    "dark:bg-pink-700",
    "dark:border-l-white",
  ],
  plugins: [],
};
