import { useGetActivities } from "~/aether-sdk";

export const ActivityHeatmap = () => {
	const { data: activities } = useGetActivities();

	return <div>ActivityHeatmap</div>;
};
