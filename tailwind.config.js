/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./src/**/*.{html,js,svelte,ts}",
  ],
  theme: {
    extend: {},
  },
  darkMode: 'class',
  plugins: [
    require('@skeletonlabs/skeleton')({
      themes: { preset: [ "skeleton" ] } // 加载预设主题
    })
  ]
}
