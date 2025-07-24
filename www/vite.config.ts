import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
    plugins: [tailwindcss()],
    base: "./",
    server: {
        fs: {
            // Allow serving files from the repository root
            allow: [".."],
        },
    },
});
