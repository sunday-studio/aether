import { useForm } from "@tanstack/react-form";
import { useQueryClient } from "@tanstack/react-query";
import { Plus } from "lucide-react";
import { Button, Dialog, DialogTrigger, Form } from "react-aria-components";
import { z } from "zod";
import { getGetGoalsQueryKey, useCreateGoal } from "~/aether-sdk";
import { Modal, modalContentStyles } from "~/components/shared/modal";
import { TextAreaField, TextField } from "~/components/shared/text-field";
import { cn } from "~/utils/cn";
import { TaskActionButton } from "../task-item/task-shared-components";

const createGoalSchema = z.object({
	name: z.string().min(1, { message: "Name is required" }),
	description: z.string().optional(),
	recurrenceType: z.enum(["daily", "weekly", "monthly", "custom"]),
	recurrenceInterval: z.number().min(1),
	recurrenceAnchor: z.date(),
	recurrenceMeta: z.record(z.string(), z.any()).optional(),
});

const inputStyles = cn(`
  w-full rounded-md bg-neutral-100 text-left align-middle text-neutral-700 shadow-2xl bg-clip-padding bg-white
`);

export const CreateGoalDialog = () => {
	const queryClient = useQueryClient();
	const { mutate, isPending, reset } = useCreateGoal({});

	const form = useForm({
		defaultValues: {
			name: "",
			description: "",
			recurrenceType: "daily",
			recurrenceInterval: 1,
			recurrenceAnchor: new Date(),
			recurrenceMeta: {},
		},
		validators: {
			onChange: (value) => createGoalSchema.safeParse(value),
		},
		onSubmit: async ({ value }) => {
			console.log("value ->", value);

			mutate(
				{
					data: {
						name: value.name,
						description: value.description,
						recurrenceType: value.recurrenceType,
						recurrenceInterval: value.recurrenceInterval,
						recurrenceAnchor: value.recurrenceAnchor,
						recurrenceMeta: value.recurrenceMeta,
					},
				},
				{
					onSuccess: () => {
						queryClient.invalidateQueries({ queryKey: getGetGoalsQueryKey() });
					},
				},
			);
		},
	});

	const handleDialogClose = () => {
		form.reset();
		reset();
	};

	return (
		<DialogTrigger>
			<Button>
				<TaskActionButton>
					<Plus size={14} strokeWidth={3} />
				</TaskActionButton>
			</Button>
			<Modal isDismissable>
				<Dialog className={modalContentStyles}>
					<p>Let's create a new goal</p>
					<Form
						className="space-y-4"
						onSubmit={(e) => {
							e.preventDefault();
							form.handleSubmit();
						}}
					>
						<form.Field name="name">
							{(field) => (
								<TextField
									autoFocus
									name={field.name}
									label="Name"
									placeholder="Enter goal name"
									value={field.state.value}
									onChange={field.handleChange}
									errorMessage={field.state.meta.errors[0]}
									// isDisabled={form.state.isSubmitting || isPending}
								/>
							)}
						</form.Field>
						<form.Field name="description">
							{(field) => (
								<TextAreaField
									name={field.name}
									label="Description"
									placeholder="Enter a description (optional)"
									value={field.state.value}
									onChange={field.handleChange}
									errorMessage={field.state.meta.errors[0]}
									// isDisabled={form.state.isSubmitting || isPending}
								/>
							)}
						</form.Field>
						<form.Field name="recurrenceType">
							{(field) => (
								<div className="mb-3">
									<label
										className="block text-sm font-medium mb-1"
										htmlFor={field.name}
									>
										Recurrence Type
									</label>
									<select
										name={field.name}
										id={field.name}
										value={field.state.value}
										onChange={(e) => field.handleChange(e.target.value)}
										className={inputStyles}
										// disabled={form.state.isSubmitting || isPending}
									>
										<option value="daily">Daily</option>
										<option value="weekly">Weekly</option>
										<option value="monthly">Monthly</option>
										<option value="custom">Custom</option>
									</select>
									{field.state.meta.errors[0] && (
										<p className="text-xs text-red-600 mt-1">
											{field.state.meta.errors[0]}
										</p>
									)}
								</div>
							)}
						</form.Field>
						<form.Field name="recurrenceInterval">
							{(field) => (
								<TextField
									type="number"
									name={field.name}
									label="Recurrence Interval"
									placeholder="e.g. 1"
									value={field.state.value}
									onChange={(e) =>
										field.handleChange(
											typeof e === "string"
												? Number(e)
												: typeof e?.target?.value !== "undefined"
													? Number(e.target.value)
													: e,
										)
									}
									errorMessage={field.state.meta.errors[0]}
									min={1}
									// isDisabled={form.state.isSubmitting || isPending}
								/>
							)}
						</form.Field>
						<form.Field name="recurrenceAnchor">
							{(field) => (
								<TextField
									type="date"
									name={field.name}
									label="Recurrence Anchor"
									placeholder="Select date"
									value={
										field.state.value instanceof Date
											? field.state.value.toISOString().slice(0, 10)
											: field.state.value
									}
									onChange={(e) => {
										const val =
											typeof e === "string"
												? e
												: e?.target?.value
													? e.target.value
													: e;
										field.handleChange(new Date(val));
									}}
									errorMessage={field.state.meta.errors[0]}
									// isDisabled={form.state.isSubmitting || isPending}
								/>
							)}
						</form.Field>
						<form.Field name="recurrenceMeta">
							{(field) => (
								<TextAreaField
									name={field.name}
									label="Recurrence Meta (JSON)"
									placeholder="Add recurrence metadata (advanced, must be JSON)"
									value={
										typeof field.state.value === "object"
											? JSON.stringify(field.state.value, null, 2)
											: field.state.value
									}
									onChange={(val) => {
										try {
											const parsed = JSON.parse(val);
											field.handleChange(parsed);
										} catch {
											field.handleChange(val);
										}
									}}
									errorMessage={field.state.meta.errors[0]}
									// isDisabled={form.state.isSubmitting || isPending}
									minRows={3}
								/>
							)}
						</form.Field>
						Tag IDs field (example, this will be a comma separated text for now)
						<form.Field name="tagIds">
							{(field) => (
								<TextField
									name={field.name}
									label="Tag IDs"
									placeholder="Comma-separated tag IDs (optional)"
									value={field.state.value ?? ""}
									onChange={(val) => {
										const arr =
											typeof val === "string"
												? val
														.split(",")
														.map((v) => v.trim())
														.filter(Boolean)
												: [];
										field.handleChange(arr);
									}}
									errorMessage={field.state.meta.errors[0]}
									// isDisabled={form.state.isSubmitting || isPending}
								/>
							)}
						</form.Field>
						<div className="flex gap-2 self-end mt-4">
							<Button
								slot="close"
								type="button"
								className="bg-neutral-200"
								onPress={handleDialogClose}
								isDisabled={form.state.isSubmitting || isPending}
							>
								Cancel
							</Button>
							<Button
								type="submit"
								className="bg-neutral-700 text-neutral-200"
								isDisabled={form.state.isSubmitting || isPending}
							>
								{isPending || form.state.isSubmitting
									? "Creating..."
									: "Create Goal"}
							</Button>
						</div>
					</Form>
				</Dialog>
			</Modal>
		</DialogTrigger>
	);
};
