// Service worker: the only JavaScript in the project.
// Cache-first strategy: the app works offline from the first visit.
// VERSION and PRECACHE are filled at build time by tools/sw-precache.sh (Trunk hook):
// the version changes whenever the build content changes, which renews the cache.
const VERSION = "7f4a72482fec3d21";
const PRECACHE = ["base-2c7d3097a7e4e03a.css","fonts/barlow-condensed-500.woff2","fonts/barlow-condensed-600.woff2","fonts/barlow-condensed-700.woff2","icons/apple-touch-icon.png","icons/icon-192.png","icons/icon-512.png","index.html","manifest.json","style-1617e1a12f33cefc.css","style-21827311e3b4ae74.css","style-24769e184c7b06fd.css","style-3348abb28d73cb98.css","style-8b1818949edf7e37.css","style-aa201b0aeaace84f.css","style-fc00f31fb184bd79.css","ui_leptos-5a88439177b6fff3.js","ui_leptos-b2f740733c199cd9_bg.wasm",];
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
