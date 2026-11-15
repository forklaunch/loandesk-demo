import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  // Sent with browser error reports, so an error can be tied to a build.
  define: { __APP_VERSION__: JSON.stringify(process.env.npm_package_version ?? "dev") },
});
