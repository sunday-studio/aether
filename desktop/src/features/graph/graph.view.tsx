import { useQuery } from "@tanstack/react-query";
import { Loader } from "lucide-react";
import { getAllLinksForGraph } from "~/aether-sdk";
import { GraphVisualization } from "./components/graph-visualization";

export const GraphView = () => {
	const { data: response, isLoading, error } = useQuery({
		queryKey: ["graphLinks"],
		queryFn: async () => {
			return getAllLinksForGraph();
		},
	});

	if (error) {
		return (
			<div className="h-full flex items-center justify-center">
				<p className="text-sm text-neutral-500">Error loading graph data</p>
			</div>
		);
	}

	if (isLoading) {
		return (
			<div className="h-full flex items-center justify-center">
				<Loader className="w-4 h-4 animate-spin" />
			</div>
		);
	}

	const links = response?.status === 200 ? response.data : [];

	return (
		<div className="h-full flex flex-col">
			<div className="flex items-center justify-between py-4">
				<h3 className="font-gt-ultra text-2xl font-medium">Knowledge Graph</h3>
			</div>
			<div className="flex-1 overflow-hidden">
				<GraphVisualization links={links} />
			</div>
		</div>
	);
};
