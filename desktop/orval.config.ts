import dotenv from "dotenv";
import { defineConfig } from "orval";

dotenv.config();


console.log("API_URL", process.env.API_URL);

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
			baseUrl: process.env.API_URL,
		},
	},
});

// http://nowhere.local:9119/v1/ping
