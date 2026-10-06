import { defineConfig } from "vite";
import handler from "./api/leaderboard.js";

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
  plugins: [
    {
      name: "api-leaderboard-middleware",
      configureServer(server) {
        server.middlewares.use(async (req, res, next) => {
          if (req.url?.startsWith("/api/leaderboard")) {
            if (req.method === "POST" && !req.body) {
              const buffers = [];
              for await (const chunk of req) {
                buffers.push(chunk);
              }
              const data = Buffer.concat(buffers).toString();
              try {
                req.body = JSON.parse(data);
              } catch (_) {
                req.body = {};
              }
            }
            await handler(req, res);
            return;
          }
          next();
        });
      },
    },
  ],
});
