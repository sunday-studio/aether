import { useQuery } from "@tanstack/react-query";
import { Bookmark, FileText, Goal, Link2, Square } from "lucide-react";
import { useNavigate } from "react-router";
import { getBacklinks } from "~/aether-sdk";
import { Popover, PopoverContent, PopoverTrigger } from "./popover";

interface BacklinksPopoverProps {
	targetType: string;
	targetId: string;
	children: React.ReactNode;
}

export function BacklinksPopover({
	targetType,
	targetId,
	children,
}: BacklinksPopoverProps) {
	const navigate = useNavigate();

	const { data: backlinks, isLoading } = useQuery({
		queryKey: ["backlinks", targetType, targetId],
		queryFn: async () => {
			return getBacklinks({
				targetType,
				targetId,
			});
		},
	});

	const getIcon = (resourceType: string) => {
		switch (resourceType) {
			case "entry":
				return <FileText size={14} />;
			case "task":
				return <Square size={14} />;
			case "goal":
				return <Goal size={14} />;
			case "canvas":
				return <Square size={14} />;
			case "bookmark":
				return <Bookmark size={14} />;
			default:
				return <Link2 size={14} />;
		}
	};

	const handleNavigate = (sourceType: string, sourceId: string) => {
		switch (sourceType) {
			case "entry":
				navigate(`/entry/${sourceId}`);
				break;
			case "task":
				navigate(`/tasks/${sourceId}`);
				break;
			case "goal":
				navigate(`/tasks/goal/${sourceId}`);
				break;
			case "canvas":
				navigate(`/canvas/${sourceId}`);
				break;
			case "bookmark":
				navigate(`/bookmarks/${sourceId}`);
				break;
			default:
				break;
		}
	};

	return (
		<Popover>
			<PopoverTrigger asChild>{children}</PopoverTrigger>
			<PopoverContent className="w-80 p-0" align="start">
				<div className="p-3 border-b border-stone-200 dark:border-stone-700">
					<h3 className="text-sm font-semibold text-stone-900 dark:text-stone-100">
						Backlinks
					</h3>
					<p className="text-xs text-stone-500 dark:text-stone-400 mt-0.5">
						Resources linking to this
					</p>
				</div>
				<div className="max-h-[300px] overflow-y-auto">
					{isLoading ? (
						<div className="p-4 text-center text-sm text-stone-500">
							Loading...
						</div>
					) : !backlinks?.data || backlinks.data.length === 0 ? (
						<div className="p-4 text-center text-sm text-stone-500">
							No backlinks found
						</div>
					) : (
						<ul className="divide-y divide-stone-200 dark:divide-stone-700">
							{backlinks.data.map((backlink) => (
								<li
									key={backlink.link.id}
									className="p-3 hover:bg-stone-50 dark:hover:bg-stone-800 cursor-pointer transition-colors"
									onClick={() =>
										handleNavigate(
											backlink.link.sourceType,
											backlink.link.sourceId,
										)
									}
								>
									<div className="flex items-start gap-2">
										<div className="text-stone-400 dark:text-stone-500 mt-0.5 flex-shrink-0">
											{getIcon(backlink.link.sourceType)}
										</div>
										<div className="flex-1 min-w-0">
											<p className="text-sm font-medium text-stone-900 dark:text-stone-100 truncate">
												{backlink.sourceTitle ||
													`${backlink.link.sourceType}:${backlink.link.sourceId}`}
											</p>
											{backlink.link.linkText && (
												<p className="text-xs text-stone-500 dark:text-stone-400 mt-0.5 truncate">
													{backlink.link.linkText}
												</p>
											)}
										</div>
									</div>
								</li>
							))}
						</ul>
					)}
				</div>
			</PopoverContent>
		</Popover>
	);
}
