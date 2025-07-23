import { defineConfig } from 'vite'

export default defineConfig({
  // Add this server configuration
  server: {
    fs: {
      // Allow serving files from the repository root
      allow: ['..']
    }
  }
})
