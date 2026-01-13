import dotenv from "dotenv";
import { defineConfig } from "orval";

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
			override: {
				mutator: {
					path: "./src/lib/api-client.ts",
					name: "customFetch",
				},
			},
		},
	},
});
