import { fetch as tauriFetch } from '@tauri-apps/plugin-http';

export async function fetch(url, init) {
    const res = await tauriFetch(url, init);
    const contentType = res.headers.get('content-type') || '';
    try {
        if (contentType.includes('application/json')) {
            res.data = await res.json();
        } else {
            res.data = await res.text();
        }
    } catch {
        res.data = null;
    }
    return res;
}
