export type RestLedgerEntry = {
	id: number;
	requestId: string;
	at: string;
	method: string;
	url: string;
	command: string;
	status: 'ok' | 'error';
	statusCode?: number;
	totalMs: number;
	routeMs: number;
	parseMs: number;
	argsMs: number;
	invokeMs: number;
	bodyBytes: number;
	errorMessage?: string;
};

declare global {
	interface Window {
		aetherRestLedgerEntries?: RestLedgerEntry[];
		aetherRestLedger?: {
			entries: () => RestLedgerEntry[];
			slow: (thresholdMs?: number) => RestLedgerEntry[];
			summary: () => Record<string, { count: number; avgMs: number; maxMs: number }>;
			clear: () => void;
		};
	}
}

const REST_LEDGER_KEY = 'aether:rest-ledger:v1';
const MAX_LEDGER_ENTRIES = 300;
const SLOW_REQUEST_THRESHOLD_MS = 150;
const REDACTED = '[REDACTED]';

let nextRestLedgerId = 1;

function loadLedger() {
	if (typeof window === 'undefined') return [];
	if (window.aetherRestLedgerEntries) return window.aetherRestLedgerEntries;

	try {
		const value = window.localStorage.getItem(REST_LEDGER_KEY);
		window.aetherRestLedgerEntries = value ? JSON.parse(value) : [];
	} catch {
		window.aetherRestLedgerEntries = [];
	}

	return window.aetherRestLedgerEntries;
}

function persistLedger(entries: RestLedgerEntry[]) {
	if (typeof window === 'undefined') return;
	try {
		window.localStorage.setItem(REST_LEDGER_KEY, JSON.stringify(entries));
	} catch {
		// Timing diagnostics should never break app behavior.
	}
}

function installLedgerHelpers() {
	if (typeof window === 'undefined' || window.aetherRestLedger) return;
	window.aetherRestLedger = {
		entries: () => [...loadLedger()],
		slow: (thresholdMs = SLOW_REQUEST_THRESHOLD_MS) =>
			loadLedger().filter(entry => entry.totalMs >= thresholdMs),
		summary: () => {
			const summary: Record<string, { count: number; avgMs: number; maxMs: number }> = {};
			for (const entry of loadLedger()) {
				const current = summary[entry.command] ?? { count: 0, avgMs: 0, maxMs: 0 };
				const total = current.avgMs * current.count + entry.totalMs;
				const count = current.count + 1;
				summary[entry.command] = {
					count,
					avgMs: Math.round((total / count) * 10) / 10,
					maxMs: Math.max(current.maxMs, entry.totalMs),
				};
			}
			return summary;
		},
		clear: () => {
			window.aetherRestLedgerEntries = [];
			window.localStorage.removeItem(REST_LEDGER_KEY);
		},
	};
}

export function recordRestLedgerEntry(entry: Omit<RestLedgerEntry, 'id' | 'at'>) {
	if (typeof window === 'undefined') return;
	installLedgerHelpers();

	const normalizedEntry: RestLedgerEntry = {
		...entry,
		id: nextRestLedgerId++,
		at: new Date().toISOString(),
		url: sanitizeUrlForLedger(entry.url),
		errorMessage: entry.errorMessage ? redactText(entry.errorMessage) : undefined,
		totalMs: Math.round(entry.totalMs * 10) / 10,
		routeMs: Math.round(entry.routeMs * 10) / 10,
		parseMs: Math.round(entry.parseMs * 10) / 10,
		argsMs: Math.round(entry.argsMs * 10) / 10,
		invokeMs: Math.round(entry.invokeMs * 10) / 10,
	};

	const entries = loadLedger();
	entries.push(normalizedEntry);
	if (entries.length > MAX_LEDGER_ENTRIES) {
		entries.splice(0, entries.length - MAX_LEDGER_ENTRIES);
	}
	persistLedger(entries);

	const logPayload = {
		id: normalizedEntry.id,
		requestId: normalizedEntry.requestId,
		method: normalizedEntry.method,
		url: normalizedEntry.url,
		command: normalizedEntry.command,
		status: normalizedEntry.status,
		totalMs: normalizedEntry.totalMs,
		routeMs: normalizedEntry.routeMs,
		parseMs: normalizedEntry.parseMs,
		argsMs: normalizedEntry.argsMs,
		invokeMs: normalizedEntry.invokeMs,
		bodyBytes: normalizedEntry.bodyBytes,
		errorMessage: normalizedEntry.errorMessage,
	};

	if (normalizedEntry.totalMs >= SLOW_REQUEST_THRESHOLD_MS || normalizedEntry.status === 'error') {
		console.warn('[REST-TIMING]', logPayload);
	} else if (import.meta.env.DEV) {
		console.debug('[REST-TIMING]', logPayload);
	}
}

export function getRestLedgerEntriesForExport(): RestLedgerEntry[] {
	if (typeof window === 'undefined') return [];
	installLedgerHelpers();
	return loadLedger().map(entry => ({
		...entry,
		url: sanitizeUrlForLedger(entry.url),
		errorMessage: entry.errorMessage ? redactText(entry.errorMessage) : undefined,
	}));
}

function sanitizeUrlForLedger(url: string): string {
	const [pathWithQuery = '', hash = ''] = url.split('#', 2);
	const [path = '', query = ''] = pathWithQuery.split('?', 2);
	if (!query) return path;

	const redactedQuery = query
		.split('&')
		.map(pair => {
			const [key = '', value = ''] = pair.split('=', 2);
			if (!value) return key;
			if (isSafeQueryValue(key, value)) return `${key}=${value}`;
			return `${key}=${REDACTED}`;
		})
		.join('&');

	return hash ? `${path}?${redactedQuery}#${hash}` : `${path}?${redactedQuery}`;
}

function isSafeQueryValue(key: string, value: string): boolean {
	const normalizedKey = key.toLowerCase();
	if (
		normalizedKey.includes('token') ||
		normalizedKey.includes('secret') ||
		normalizedKey.includes('password') ||
		normalizedKey.includes('passphrase') ||
		normalizedKey.includes('api_key') ||
		normalizedKey.includes('apikey')
	) {
		return false;
	}
	return ['limit', 'offset', 'page', 'cursor'].includes(normalizedKey) && value.length <= 64;
}

function redactText(value: string): string {
	return value
		.replace(
			/(authorization|password|passphrase|token|secret|api_key|apikey)=([^&,"'\r\n]+)/gi,
			`$1=${REDACTED}`,
		)
		.replace(/\b(phx_[a-z0-9_]+|sk-[a-z0-9_-]+)\b/gi, REDACTED);
}
