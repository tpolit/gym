// Service worker: the only JavaScript in the project.
// Cache-first strategy: the app works offline from the first visit.
// VERSION and PRECACHE are filled at build time by tools/sw-precache.sh (Trunk hook):
// the version changes whenever the build content changes, which renews the cache.
const VERSION = "__SW_VERSION__";
const PRECACHE = [/* __SW_PRECACHE__ */];
// Specific prefix: caches are shared by every app under tpolit.github.io, and old ones are deleted by prefix.
const CACHE_PREFIX = "gym-";
const CACHE = CACHE_PREFIX + VERSION;

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(CACHE)
      // `reload` bypasses the HTTP cache so that an old version is not precached.
      .then((cache) => cache.addAll(["./", ...PRECACHE].map((url) => new Request(url, { cache: "reload" }))))
      .then(() => self.skipWaiting()),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(keys.filter((key) => key.startsWith(CACHE_PREFIX) && key !== CACHE).map((key) => caches.delete(key))),
      )
      .then(() => self.clients.claim()),
  );
});

self.addEventListener("fetch", (event) => {
  const request = event.request;
  if (request.method !== "GET" || new URL(request.url).origin !== self.location.origin) {
    return;
  }
  event.respondWith(
    caches.open(CACHE).then(async (cache) => {
      const cached = await cache.match(request, { ignoreSearch: request.mode === "navigate" });
      if (cached) {
        return cached;
      }
      const response = await fetch(request);
      if (response.ok) {
        cache.put(request, response.clone());
      }
      return response;
    }),
  );
});
