import { Button } from "~/components/shared/button";
import { TextField } from "~/components/shared/text-field";

export const SyncSection = () => {
	return (
		<div className="space-y-10 max-w-xl">
			<div>
				<h3 className="text-lg font-medium">Sync</h3>
				<p className="text-sm text-(--color-secondary-text)">
					Setup a LibSQL remote replica to sync your data across devices. Read
					more about how to setup one{" "}
					<a
						href="https://github.com/sunday-studio/aether/blob/main/remote-replica/README.md"
						target="_blank"
						rel="noopener noreferrer"
					>
						here
					</a>
					.
				</p>
			</div>

			<div className="flex flex-col gap-4">
				<TextField
					label="Remote URL"
					placeholder="https://your-remote-url.com"
				/>
				<TextField
					label="Authentication token"
					placeholder="your-authentication-token"
				/>

				<div className="flex items-center justify-end gap-4 mt-4">
					<Button
						onClick={(): void => {
							throw new Error("Function not implemented.");
						}}
						label="Remove config"
						variant="destructive"
						tooltipContent="Remove the remote database configuration"
					/>

					<Button
						onClick={(): void => {
							throw new Error("Function not implemented.");
						}}
						label="Save change"
						tooltipContent="Save the changes to the remote database"
					/>
				</div>
			</div>
		</div>
	);
};
