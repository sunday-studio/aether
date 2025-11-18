import dotenv from "dotenv";
import {defineConfig} from "orval";

dotenv.config();
export default defineConfig({
  "aether-sdk": {
    input: "../backend/docs/swagger.json",
    output: {
      namingConvention: "kebab-case",
      clean: true,
      target: "./src/aether-sdk/index.ts",
      mode: "split",
      schemas: "./src/aether-sdk/models",
      client: "react-query",
      mock: false,
      baseUrl: "http://127.0.0.1:9119/v1",
    },
  },
});
