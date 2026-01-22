import { useNavigate } from "react-router";
import { cn } from "~/utils/cn";

interface ResourceLinkProps {
	targetType: string;
	targetId: string;
	linkText: string | null;
	className?: string;
}

export function ResourceLink({
	targetType,
	targetId,
	linkText,
	className,
}: ResourceLinkProps) {
	const navigate = useNavigate();

	const handleClick = (e: React.MouseEvent) => {
		e.preventDefault();
		// Navigate to the resource based on type
		switch (targetType) {
			case "entry":
				navigate(`/entry/${targetId}`);
				break;
			case "task":
				navigate(`/tasks/${targetId}`);
				break;
			case "goal":
				navigate(`/tasks/goal/${targetId}`);
				break;
			case "canvas":
				navigate(`/canvas/${targetId}`);
				break;
			case "bookmark":
				navigate(`/bookmarks/${targetId}`);
				break;
			default:
				break;
		}
	};

	const displayText = linkText || `${targetType}:${targetId}`;

	return (
		<span
			className={cn(
				"inline-flex items-center px-1.5 py-0.5 rounded text-sm font-medium",
				"bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
				"hover:bg-blue-200 dark:hover:bg-blue-800",
				"cursor-pointer transition-colors",
				className,
			)}
			onClick={handleClick}
			role="button"
			tabIndex={0}
			onKeyDown={(e) => {
				if (e.key === "Enter" || e.key === " ") {
					e.preventDefault();
					handleClick(e as unknown as React.MouseEvent);
				}
			}}
		>
			{displayText}
		</span>
	);
}
