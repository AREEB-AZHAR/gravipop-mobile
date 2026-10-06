import { defineConfig } from "vite";

export default defineConfig({
  root: "./",
  publicDir: "public",
  server: {
    port: 3000,
    host: true,
  },
  preview: {
    port: 8080,
    host: true,
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    target: "esnext",
  },
});
