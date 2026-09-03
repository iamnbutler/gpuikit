/*
 * Cross-origin isolation on hosts that cannot set response headers.
 *
 * gpui's web platform runs its background executor on web workers over a
 * shared wasm memory, and SharedArrayBuffer is only available when the page
 * is cross-origin isolated (COOP + COEP response headers). GitHub Pages
 * cannot send custom headers, so this file does double duty:
 *
 *  - loaded as a page <script>, it registers itself as a service worker and
 *    reloads the page once so the worker takes control;
 *  - running as that service worker, it re-serves every same-scope fetch
 *    with the two isolation headers attached.
 *
 * It is a no-op wherever real headers already exist (e.g. `trunk serve`,
 * which sends them itself — see trunk.toml).
 */

if (typeof window !== "undefined") {
    // Page context.
    if (!window.crossOriginIsolated && "serviceWorker" in navigator) {
        const swUrl = document.currentScript.src;
        navigator.serviceWorker
            .register(swUrl)
            .then(() => {
                if (navigator.serviceWorker.controller) return;
                // First visit: reload once the worker has claimed the page so
                // the app boots with SharedArrayBuffer available.
                navigator.serviceWorker.addEventListener(
                    "controllerchange",
                    () => window.location.reload(),
                    { once: true },
                );
            })
            .catch((err) =>
                console.error("coi-serviceworker: registration failed", err),
            );
    }
} else {
    // Service-worker context.
    self.addEventListener("install", () => self.skipWaiting());
    self.addEventListener("activate", (event) =>
        event.waitUntil(self.clients.claim()),
    );
    self.addEventListener("fetch", (event) => {
        const request = event.request;
        if (request.cache === "only-if-cached" && request.mode !== "same-origin") {
            return;
        }
        event.respondWith(
            fetch(request).then((response) => {
                if (response.status === 0) return response;
                const headers = new Headers(response.headers);
                headers.set("Cross-Origin-Embedder-Policy", "require-corp");
                headers.set("Cross-Origin-Opener-Policy", "same-origin");
                return new Response(response.body, {
                    status: response.status,
                    statusText: response.statusText,
                    headers,
                });
            }),
        );
    });
}
