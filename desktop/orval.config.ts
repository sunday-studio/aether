import dotenv from 'dotenv';
import { defineConfig } from 'orval';
import { readFileSync } from 'node:fs';

dotenv.config();

// Paginated endpoints that support cursor-based infinite scroll
const infiniteQueryConfig = {
	useQuery: true,
	useInfinite: true,
	useInfiniteQueryParam: 'cursor',
};

const mutationQueryConfig = {
	useQuery: false,
	useMutation: true,
	useInfinite: false,
};

type OpenApiOperation = {
	operationId?: string;
};

type OpenApiSpec = {
	paths: Record<string, Record<string, OpenApiOperation>>;
};

const spec = JSON.parse(
	readFileSync(new URL('./src/openapi/spec.json', import.meta.url), 'utf8'),
) as OpenApiSpec;

const writeMethods = new Set(['delete', 'patch', 'post', 'put']);

const writeOperationConfig = Object.fromEntries(
	Object.values(spec.paths).flatMap(pathItem =>
		Object.entries(pathItem)
			.filter(([method, operation]) => writeMethods.has(method) && Boolean(operation.operationId))
			.map(([, operation]) => [operation.operationId, { query: mutationQueryConfig }]),
	),
);

export default defineConfig({
	'aether-sdk': {
		input: './src/openapi/spec.json',
		output: {
			namingConvention: 'kebab-case',
			clean: true,
			target: './src/aether-sdk/index.ts',
			mode: 'split',
			schemas: './src/aether-sdk/models',
			client: 'react-query',
			mock: false,
			override: {
				mutator: {
					path: './src/lib/api-client.ts',
					name: 'customFetch',
				},
				// Default: only useQuery, no infinite scroll
				query: {
					useQuery: true,
					useMutation: false,
					useInfinite: false,
				},
				// Enable infinite scroll only for paginated endpoints
				operations: {
					...writeOperationConfig,
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
