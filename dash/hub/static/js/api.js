// One way to talk to the hub: every request is uncached and fails loudly on a non-2xx.

const API = "/api/v1";

async function request(path, init) {
  const url = /^(\/|https?:)/.test(path) ? path : `${API}/${path}`;
  const res = await fetch(url, { cache: "no-store", ...init });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    const err = new Error(body.error || `${init && init.method || "GET"} ${url} → ${res.status}`);
    err.status = res.status;
    throw err;
  }
  return res.json();
}

const query = params => {
  const q = new URLSearchParams();
  for (const [k, v] of Object.entries(params || {})) if (v != null && v !== "") q.set(k, String(v));
  const s = q.toString();
  return s ? "?" + s : "";
};

export const getJson = (path, params) => request(path + query(params));

/** Like getJson, but resolves to `fallback` instead of throwing. */
export const tryJson = (path, params, fallback = null) => getJson(path, params).catch(() => fallback);

export const post = (path, body) => request(path, {
  method: "POST",
  headers: body ? { "content-type": "application/json" } : undefined,
  body: body ? JSON.stringify(body) : undefined,
});

export const enc = encodeURIComponent;
