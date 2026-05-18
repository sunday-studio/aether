#!/usr/bin/env node

import crypto from "node:crypto";
import { performance } from "node:perf_hooks";

const baseUrl = (process.env.SYNC_SERVER_URL ?? "http://localhost:8080").replace(/\/$/, "");
const serverSeedPhrase = process.env.SERVER_SEED_PHRASE ?? process.env.SERVER_PASSPHRASE;
const changeCount = Number.parseInt(process.env.CHANGES ?? "500", 10);
const deviceId = `bench-${crypto.randomUUID()}`;
const hostname = "sync-benchmark";

if (!serverSeedPhrase || serverSeedPhrase.trim().length < 12) {
	console.error("SERVER_SEED_PHRASE must be set and at least 12 characters.");
	process.exit(1);
}

if (!Number.isFinite(changeCount) || changeCount < 1) {
	console.error("CHANGES must be a positive integer.");
	process.exit(1);
}

async function timed(label, fn) {
	const started = performance.now();
	const result = await fn();
	const elapsedMs = performance.now() - started;
	console.log(`${label}: ${elapsedMs.toFixed(1)}ms`);
	return result;
}

async function request(path, options = {}) {
	const response = await fetch(`${baseUrl}${path}`, options);
	if (!response.ok) {
		const body = await response.text().catch(() => "");
		throw new Error(`${options.method ?? "GET"} ${path} failed ${response.status}: ${body}`);
	}
	return response;
}

function encryptedChange(index) {
	return {
		nonce: crypto.randomBytes(12).toString("base64"),
		ciphertext: Buffer.from(
			JSON.stringify({
				entity: "entries",
				id: `bench-entry-${index}`,
				op: "upsert",
				updated_at: Date.now(),
				data: { id: `bench-entry-${index}`, document: "benchmark" },
			}),
		).toString("base64"),
	};
}

const enrollment = await timed("register", async () => {
	const response = await request("/register", {
		method: "POST",
		headers: { "content-type": "application/json" },
		body: JSON.stringify({
			device_id: deviceId,
			hostname,
			server_seed_phrase: serverSeedPhrase,
		}),
	});
	return response.json();
});

const authHeaders = {
	authorization: `Bearer ${enrollment.device_token}`,
	"x-aether-device-id": deviceId,
};

const changes = Array.from({ length: changeCount }, (_, index) => encryptedChange(index));

await timed(`push ${changeCount} changes`, async () => {
	await request("/push", {
		method: "POST",
		headers: {
			...authHeaders,
			"content-type": "application/json",
		},
		body: JSON.stringify({
			batch_id: crypto.randomUUID(),
			device_hostname: hostname,
			changes,
		}),
	});
});

await timed("pull all pages", async () => {
	let cursor;
	let total = 0;
	let pages = 0;
	do {
		const path = cursor ? `/pull?cursor=${encodeURIComponent(cursor)}` : "/pull";
		const response = await request(path, { headers: authHeaders });
		const body = await response.json();
		pages += 1;
		total += body.changes.length;
		cursor = body.next_cursor
			? `${body.next_cursor.received_at}:${body.next_cursor.change_id}`
			: undefined;
		if (!body.has_more) {
			break;
		}
	} while (cursor);

	console.log(`pulled changes: ${total}`);
	console.log(`pull pages: ${pages}`);
});
