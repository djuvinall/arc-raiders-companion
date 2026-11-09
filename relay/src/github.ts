// GitHub App auth and the few REST calls the relay makes.

export interface GitHubEnv {
  GITHUB_APP_ID: string;
  /** The .pem GitHub gives you, PKCS#1 ("BEGIN RSA PRIVATE KEY") or PKCS#8. */
  GITHUB_PRIVATE_KEY: string;
  /** "owner/name" of the data repository. */
  GITHUB_REPO: string;
  /** Overridable for tests. */
  GITHUB_API?: string;
}

const enc = new TextEncoder();

function b64url(bytes: Uint8Array | string): string {
  const b = typeof bytes === "string" ? enc.encode(bytes) : bytes;
  let s = "";
  for (const x of b) s += String.fromCharCode(x);
  return btoa(s).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

export function b64(text: string): string {
  let s = "";
  for (const x of enc.encode(text)) s += String.fromCharCode(x);
  return btoa(s);
}

function derLength(n: number): number[] {
  if (n < 0x80) return [n];
  const out: number[] = [];
  while (n > 0) {
    out.unshift(n & 0xff);
    n >>= 8;
  }
  return [0x80 | out.length, ...out];
}

function der(tag: number, body: Uint8Array): Uint8Array {
  const len = derLength(body.length);
  const out = new Uint8Array(1 + len.length + body.length);
  out[0] = tag;
  out.set(len, 1);
  out.set(body, 1 + len.length);
  return out;
}

function concat(...parts: Uint8Array[]): Uint8Array {
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let at = 0;
  for (const p of parts) {
    out.set(p, at);
    at += p.length;
  }
  return out;
}

/** PEM to PKCS#8 DER. GitHub hands out PKCS#1, which WebCrypto can't import directly. */
export function pkcs8(pem: string): Uint8Array {
  const body = pem.replace(/-----[^-]+-----/g, "").replace(/\s+/g, "");
  const raw = Uint8Array.from(atob(body), (c) => c.charCodeAt(0));
  if (!pem.includes("BEGIN RSA PRIVATE KEY")) return raw;
  // PrivateKeyInfo ::= SEQUENCE { version 0, AlgorithmIdentifier rsaEncryption, OCTET STRING pkcs1 }
  const version = new Uint8Array([0x02, 0x01, 0x00]);
  const algorithm = new Uint8Array([0x30, 0x0d, 0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01, 0x05, 0x00]);
  return der(0x30, concat(version, algorithm, der(0x04, raw)));
}

export async function appJwt(env: GitHubEnv, now = Math.floor(Date.now() / 1000)): Promise<string> {
  const key = await crypto.subtle.importKey(
    "pkcs8",
    pkcs8(env.GITHUB_PRIVATE_KEY.replace(/\\n/g, "\n")),
    { name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" },
    false,
    ["sign"],
  );
  const head = b64url(JSON.stringify({ alg: "RS256", typ: "JWT" }));
  const claims = b64url(JSON.stringify({ iat: now - 60, exp: now + 540, iss: env.GITHUB_APP_ID }));
  const sig = await crypto.subtle.sign("RSASSA-PKCS1-v1_5", key, enc.encode(`${head}.${claims}`));
  return `${head}.${claims}.${b64url(new Uint8Array(sig))}`;
}

export class GitHub {
  private api: string;
  private repo: string;
  private token: string;

  private constructor(api: string, repo: string, token: string) {
    this.api = api;
    this.repo = repo;
    this.token = token;
  }

  static async connect(env: GitHubEnv): Promise<GitHub> {
    const api = env.GITHUB_API ?? "https://api.github.com";
    const jwt = await appJwt(env);
    const inst = await call<{ id: number }>(api, jwt, "GET", `/repos/${env.GITHUB_REPO}/installation`);
    const tok = await call<{ token: string }>(api, jwt, "POST", `/app/installations/${inst.id}/access_tokens`);
    return new GitHub(api, env.GITHUB_REPO, tok.token);
  }

  req<T>(method: string, path: string, body?: unknown): Promise<T> {
    return call<T>(this.api, this.token, method, `/repos/${this.repo}${path}`, body);
  }
}

async function call<T>(api: string, token: string, method: string, path: string, body?: unknown): Promise<T> {
  const res = await fetch(api + path, {
    method,
    headers: {
      authorization: `Bearer ${token}`,
      accept: "application/vnd.github+json",
      "x-github-api-version": "2022-11-28",
      "user-agent": "arc-companion-report-relay",
      ...(body === undefined ? {} : { "content-type": "application/json" }),
    },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!res.ok) throw new Error(`GitHub ${method} ${path}: ${res.status} ${(await res.text()).slice(0, 300)}`);
  return (await res.json()) as T;
}
