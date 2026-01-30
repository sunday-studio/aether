import dotenv from "dotenv";
import { defineConfig } from "orval";

dotenv.config();

// Paginated endpoints that support cursor-based infinite scroll
const infiniteQueryConfig = {
	useQuery: true,
	useInfinite: true,
	useInfiniteQueryParam: "cursor",
};

export default defineConfig({
	"aether-sdk": {
		input: "./src/openapi/spec.json",
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
				// Default: only useQuery, no infinite scroll
				query: {
					useQuery: true,
					useInfinite: false,
				},
				// Enable infinite scroll only for paginated endpoints
				operations: {
					get_entries: { query: infiniteQueryConfig },
					get_all_tags: { query: infiniteQueryConfig },
					get_inbox_tasks: { query: infiniteQueryConfig },
					get_overdue_tasks: { query: infiniteQueryConfig },
					get_goals: { query: infiniteQueryConfig },
					get_goal_instances: { query: infiniteQueryConfig },
					get_bookmarks: { query: infiniteQueryConfig },
					get_canvases: { query: infiniteQueryConfig },
					get_all_links_for_graph: { query: infiniteQueryConfig },
					get_transcriptions: { query: infiniteQueryConfig },
				},
			},
		},
	},
});
