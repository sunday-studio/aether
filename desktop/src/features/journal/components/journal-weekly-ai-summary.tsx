import { invoke } from '@tauri-apps/api/core';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { endOfWeek, format, startOfWeek } from 'date-fns';
import { Check, CheckCircle2, Sparkles } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { Button } from '~/components/shared/button';
import { TextAreaField } from '~/components/shared/text-field';
import { showToast } from '~/components/shared/toast-components';

type WeeklyAiSummary = {
	id: string;
	weekStart: string;
	weekEnd: string;
	summary: string;
	themes: string[];
	completedWork: string[];
	openLoops: string[];
	nextFocus: string[];
	state: string;
};

const weeklySummaryQueryKey = (startDate: string, endDate: string) =>
	['journal-weekly-ai-summary', startDate, endDate] as const;

async function getWeeklySummary(startDate: string, endDate: string) {
	return invoke<WeeklyAiSummary>('get_weekly_ai_summary', {
		queryParams: { startDate, endDate },
	});
}

async function generateWeeklySummary(startDate: string, endDate: string) {
	return invoke<WeeklyAiSummary>('generate_weekly_ai_summary', {
		requestData: { startDate, endDate, mode: 'rules' },
	});
}

async function updateWeeklySummary(
	summaryId: string,
	fields: {
		summary: string;
		themes: string[];
		completedWork: string[];
		openLoops: string[];
		nextFocus: string[];
		state?: string;
	},
) {
	return invoke<WeeklyAiSummary>('update_weekly_ai_summary', {
		requestData: {
			summaryId,
			summary: fields.summary,
			themes: fields.themes,
			completedWork: fields.completedWork,
			openLoops: fields.openLoops,
			nextFocus: fields.nextFocus,
			state: fields.state,
		},
	});
}

export function JournalWeeklyAiSummary() {
	const queryClient = useQueryClient();
	const [isOpen, setIsOpen] = useState(false);
	const [summaryDraft, setSummaryDraft] = useState('');
	const [themesDraft, setThemesDraft] = useState('');
	const [completedWorkDraft, setCompletedWorkDraft] = useState('');
	const [openLoopsDraft, setOpenLoopsDraft] = useState('');
	const [nextFocusDraft, setNextFocusDraft] = useState('');
	const weekRange = useMemo(() => {
		const now = new Date();
		const start = startOfWeek(now, { weekStartsOn: 1 });
		const end = endOfWeek(now, { weekStartsOn: 1 });
		return {
			startDate: start.toISOString(),
			endDate: end.toISOString(),
			label: `${format(start, 'MMM d')} - ${format(end, 'MMM d')}`,
		};
	}, []);

	const queryKey = weeklySummaryQueryKey(weekRange.startDate, weekRange.endDate);
	const summaryQuery = useQuery({
		queryKey,
		queryFn: () => getWeeklySummary(weekRange.startDate, weekRange.endDate),
		enabled: isOpen,
		retry: false,
	});

	useEffect(() => {
		const summary = summaryQuery.data;
		if (summary) {
			setSummaryDraft(summary.summary);
			setThemesDraft(listToText(summary.themes));
			setCompletedWorkDraft(listToText(summary.completedWork));
			setOpenLoopsDraft(listToText(summary.openLoops));
			setNextFocusDraft(listToText(summary.nextFocus));
		}
	}, [summaryQuery.data]);

	const refreshSummary = (summary: WeeklyAiSummary) => {
		queryClient.setQueryData(queryKey, summary);
		setSummaryDraft(summary.summary);
		setThemesDraft(listToText(summary.themes));
		setCompletedWorkDraft(listToText(summary.completedWork));
		setOpenLoopsDraft(listToText(summary.openLoops));
		setNextFocusDraft(listToText(summary.nextFocus));
	};

	const generateMutation = useMutation({
		mutationFn: () => generateWeeklySummary(weekRange.startDate, weekRange.endDate),
		onSuccess: summary => {
			refreshSummary(summary);
			showToast({ title: 'Weekly summary draft ready' });
		},
	});

	const saveMutation = useMutation({
		mutationFn: () =>
			updateWeeklySummary(summaryQuery.data?.id ?? '', {
				summary: summaryDraft,
				themes: textToList(themesDraft),
				completedWork: textToList(completedWorkDraft),
				openLoops: textToList(openLoopsDraft),
				nextFocus: textToList(nextFocusDraft),
			}),
		onSuccess: summary => {
			refreshSummary(summary);
			showToast({ title: 'Weekly summary updated' });
		},
	});

	const markReviewedMutation = useMutation({
		mutationFn: () =>
			updateWeeklySummary(summaryQuery.data?.id ?? '', {
				summary: summaryDraft,
				themes: textToList(themesDraft),
				completedWork: textToList(completedWorkDraft),
				openLoops: textToList(openLoopsDraft),
				nextFocus: textToList(nextFocusDraft),
				state: 'reviewed',
			}),
		onSuccess: summary => {
			refreshSummary(summary);
			showToast({ title: 'Weekly summary reviewed' });
		},
	});

	const hasSummary = Boolean(summaryQuery.data);

	return (
		<div className='w-full max-w-xl'>
			<div className='flex flex-wrap items-center gap-2'>
				<Button
					variant='secondary'
					label={isOpen ? 'Hide weekly AI' : 'Weekly AI'}
					tooltipContent='Toggle weekly AI summary'
					iconLeft={<Sparkles className='size-3.5' />}
					onClick={() => setIsOpen(!isOpen)}
					className='h-9 px-3 py-2 text-xs'
				/>
				<p className='text-xs text-(--color-secondary-text)'>{weekRange.label}</p>
			</div>

			{isOpen && (
				<div className='mt-3 rounded-lg border border-(--color-border) bg-(--color-panel) p-3'>
					<div className='flex flex-wrap items-center justify-between gap-2'>
						<p className='text-sm font-medium text-(--color-primary-text)'>
							Weekly summary draft
						</p>
						<Button
							variant='ghost'
							label={hasSummary ? 'Regenerate' : 'Generate'}
							tooltipContent='Generate local weekly summary draft'
							iconLeft={<Sparkles className='size-3.5' />}
							isDisabled={generateMutation.isPending}
							onClick={() => generateMutation.mutate()}
							className='h-8 px-3 py-1.5 text-xs'
						/>
					</div>

					{summaryQuery.isError && !hasSummary ? (
						<p className='mt-3 text-sm text-(--color-secondary-text)'>No draft yet.</p>
					) : null}

					{hasSummary ? (
						<div className='mt-3 space-y-3'>
							<TextAreaField
								label='Summary'
								value={summaryDraft}
								onChange={setSummaryDraft}
							/>
							<div className='grid gap-3 md:grid-cols-2'>
								<TextAreaField
									label='Themes'
									value={themesDraft}
									onChange={setThemesDraft}
								/>
								<TextAreaField
									label='Completed'
									value={completedWorkDraft}
									onChange={setCompletedWorkDraft}
								/>
								<TextAreaField
									label='Open loops'
									value={openLoopsDraft}
									onChange={setOpenLoopsDraft}
								/>
								<TextAreaField
									label='Next focus'
									value={nextFocusDraft}
									onChange={setNextFocusDraft}
								/>
							</div>
							<div className='flex justify-end gap-2'>
								<Button
									variant='secondary'
									label='Save'
									tooltipContent='Save edited weekly summary'
									iconLeft={<Check className='size-3.5' />}
									isDisabled={saveMutation.isPending || markReviewedMutation.isPending}
									onClick={() => saveMutation.mutate()}
									className='h-8 px-3 py-1.5 text-xs'
								/>
								<Button
									variant='secondary'
									label={summaryQuery.data?.state === 'reviewed' ? 'Reviewed' : 'Mark reviewed'}
									tooltipContent='Mark weekly summary as reviewed'
									iconLeft={<CheckCircle2 className='size-3.5' />}
									isDisabled={
										markReviewedMutation.isPending || summaryQuery.data?.state === 'reviewed'
									}
									onClick={() => markReviewedMutation.mutate()}
									className='h-8 px-3 py-1.5 text-xs'
								/>
							</div>
							<WeeklySummaryLists summary={summaryQuery.data!} />
						</div>
					) : null}
				</div>
			)}
		</div>
	);
}

function listToText(values: string[]) {
	return values.join('\n');
}

function textToList(value: string) {
	return value
		.split(/\n|,/)
		.map(item => item.trim())
		.filter(Boolean);
}

function WeeklySummaryLists({ summary }: { summary: WeeklyAiSummary }) {
	const groups = [
		['Themes', summary.themes],
		['Completed', summary.completedWork],
		['Open loops', summary.openLoops],
		['Next focus', summary.nextFocus],
	] as const;

	return (
		<div className='grid gap-3 md:grid-cols-2'>
			{groups
				.filter(([, values]) => values.length > 0)
				.map(([label, values]) => (
					<div key={label} className='space-y-1'>
						<p className='text-xs font-medium text-(--color-secondary-text)'>{label}</p>
						<div className='flex flex-wrap gap-1.5'>
							{values.slice(0, 6).map(value => (
								<span
									key={value}
									className='rounded-full bg-neutral-100 px-2 py-1 text-xs text-neutral-700'
								>
									{value}
								</span>
							))}
						</div>
					</div>
				))}
		</div>
	);
}
