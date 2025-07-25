import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";
import { version } from "./package.json";

export default defineConfig({
    plugins: [tailwindcss()],
    base: "./",
    server: {
        fs: {
            // Allow serving files from the repository root
            allow: [".."],
        },
    },
    define: {
        PKG_VERSION: JSON.stringify(version),
    },
});
