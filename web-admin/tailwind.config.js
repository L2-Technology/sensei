const defaultTheme = require('tailwindcss/defaultTheme')

module.exports = {
  content: ["./src/**/*.{js,jsx,ts,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter var', ...defaultTheme.fontFamily.sans],
      },
      colors: {
        orange: "#FF7353",
        'orange-hover': "#E65A3A",
        gray: {
          text: "#B4B3CD",
          background: "#221F30",
          accent1: "#615F7E",
          accent2: "#343046",
          accent3: "#3B364F",
          senseihero: "#282536",
        },
        'plum': '#282536',
        'light-plum': '#D3D2E9',
        'peach': '#FF7353',
        'smoke': '#625F7E',
        'plum-100': '#343046',
        'plum-200': '#413C56',
        'plum-50': '#443F58',
        'plum-300': '#312D41',
      },
    },
  },
  plugins: [
    require('@tailwindcss/forms'),
    require('@tailwindcss/typography'),
    require('@tailwindcss/aspect-ratio'),
  ],
}
