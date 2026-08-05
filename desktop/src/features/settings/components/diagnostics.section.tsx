import { invoke } from '@tauri-apps/api/core';
import { openPath } from '@tauri-apps/plugin-opener';
import { Download, FileJson, TriangleAlert } from 'lucide-react';
import { useState } from 'react';
import { toast } from 'sonner';
import { Button } from '~/components/shared/button';
import { getFrontendLedgerEntriesForExport } from '~/lib/performance-ledger';

type DebugLogExport = {
	path: string;
	rustEntries: number;
	frontendEntries: number;
	errorEntries: number;
};

export const DiagnosticsSection = () => {
	const [isExporting, setIsExporting] = useState(false);
	const [lastExport, setLastExport] = useState<DebugLogExport | null>(null);
	const [isOpeningErrorLog, setIsOpeningErrorLog] = useState(false);

	const exportDebugLogs = async () => {
		setIsExporting(true);
		try {
			const frontendEntries = getFrontendLedgerEntriesForExport();
			const result = await invoke<DebugLogExport>('export_debug_logs', { frontendEntries });
			setLastExport(result);
			toast.success('Debug log exported', { description: result.path });
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			toast.error('Debug log export failed', { description: message });
		} finally {
			setIsExporting(false);
		}
	};

	const openErrorLog = async () => {
		setIsOpeningErrorLog(true);
		try {
			const path = await invoke<string>('get_error_log_path');
			await openPath(path);
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			toast.error('Could not open error log', { description: message });
		} finally {
			setIsOpeningErrorLog(false);
		}
	};

	return (
		<div className='w-full space-y-6'>
			<div className='mb-10'>
				<h3 className='text-lg font-medium'>Diagnostics</h3>
				<p className='text-sm text-(--color-secondary-text)'>
					Export local timing diagnostics for support.
				</p>
			</div>

			<div className='flex items-center justify-between gap-4 border-t border-(--color-border) pt-4'>
				<div className='min-w-0'>
					<h4 className='text-md'>Live error log</h4>
					<p className='text-xs text-(--color-secondary-text)'>
						A bounded, redacted JSONL log for all command failures, panics, and error-level events.
					</p>
				</div>
				<Button
					onClick={openErrorLog}
					label={isOpeningErrorLog ? 'Opening' : 'Open log'}
					tooltipContent='Open the live error log'
					variant='secondary'
					isDisabled={isOpeningErrorLog}
					iconLeft={<TriangleAlert className='size-4' strokeWidth={2.4} />}
				/>
			</div>

			<div className='flex items-center justify-between gap-4'>
				<div className='min-w-0'>
					<h4 className='text-md'>Debug log export</h4>
					<p className='text-xs text-(--color-secondary-text)'>
						Includes redacted command timings, repository timings, and frontend API timings.
					</p>
				</div>
				<Button
					onClick={exportDebugLogs}
					label={isExporting ? 'Exporting' : 'Export'}
					tooltipContent='Export redacted debug logs'
					variant='secondary'
					isDisabled={isExporting}
					iconLeft={<Download className='size-4' strokeWidth={2.4} />}
				/>
			</div>

			{lastExport && (
				<div className='flex items-start gap-3 border-t border-(--color-border) pt-4'>
					<FileJson className='mt-0.5 size-4 shrink-0 text-(--color-secondary-text)' />
					<div className='min-w-0 space-y-1'>
						<p className='text-sm'>Last export</p>
						<p className='text-xs break-all text-(--color-secondary-text)'>{lastExport.path}</p>
						<p className='text-xs text-(--color-secondary-text)'>
							{lastExport.rustEntries} Rust entries, {lastExport.frontendEntries} frontend entries,{' '}
							{lastExport.errorEntries} error entries.
						</p>
					</div>
				</div>
			)}
		</div>
	);
};
