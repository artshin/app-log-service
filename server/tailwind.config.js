/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./templates/**/*.html",
    "./static/js/**/*.js"
  ],
  theme: {
    extend: {
      colors: {
        log: {
          trace: '#6b7280',    // gray-500
          debug: '#6b7280',    // gray-500
          info: '#10b981',     // emerald-500
          notice: '#0ea5e9',   // sky-500
          warning: '#f59e0b',  // amber-500
          error: '#ef4444',    // red-500
          critical: '#d946ef', // fuchsia-500
        }
      }
    }
  },
  plugins: []
}
