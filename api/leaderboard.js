// GraviPop Global Shared Leaderboard API (Vercel Serverless Function & Node Dev Handler)
// Synchronizes scores globally across all devices & players.

const DREAMLO_PUBLIC = "6ac4ec7f8f40bb15a8cf34f8";
const DREAMLO_PRIVATE = "BKaONkQHlU2ti8qBqP3VjAQ2-zOQJxNUq6sc1A7bwpcQ";

export async function fetchGlobalLeaderboard() {
  try {
    const res = await fetch(`http://dreamlo.com/lb/${DREAMLO_PUBLIC}/json`, {
      headers: { "Accept": "application/json" }, signal: AbortSignal.timeout(5000)
    });
    if (!res.ok) {
      throw new Error(`Leaderboard provider unavailable (HTTP ${res.status})`);
    }
    const data = await res.json();
    const rawEntries = data?.dreamlo?.leaderboard?.entry;
    if (!rawEntries) {
      return [];
    }
    const list = Array.isArray(rawEntries) ? rawEntries : [rawEntries];
    const normalized = list
      .filter(item => item && item.name && typeof item.score !== "undefined")
      .map(item => ({
        display_name: decodeURIComponent(item.name).replace(/\+/g, " ").trim(),
        high_score: Math.max(0, parseInt(item.score, 10) || 0)
      }))
      .sort((a, b) => b.high_score - a.high_score)
      .slice(0, 20);

    return normalized;
  } catch (err) {
    console.error("Leaderboard fetch error:", err);
    throw new Error("Leaderboard provider unavailable");
  }
}

export async function submitGlobalScore(name, score) {
  const cleanName = (name || "").replace(/[^\w\s.-]/g, "").trim().substring(0, 20);
  const cleanScore = Math.max(0, Math.min(1000000000, parseInt(score, 10) || 0));

  if (!cleanName || cleanName.length < 3) {
    throw new Error("Display name must be between 3 and 20 characters");
  }

  // Dreamlo spaces encoded as + or URL-safe
  const encodedName = encodeURIComponent(cleanName);
  const res = await fetch(`http://dreamlo.com/lb/${DREAMLO_PRIVATE}/add/${encodedName}/${cleanScore}`, { signal: AbortSignal.timeout(5000) });
  if (!res.ok) {
    throw new Error(`Failed to record score (HTTP ${res.status})`);
  }

  return await fetchGlobalLeaderboard();
}

export default async function handler(req, res) {
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type");

  if (req.method === "OPTIONS") {
    res.statusCode = 204;
    res.end();
    return;
  }

  if (req.method === "GET") {
    let scores;
    try { scores = await fetchGlobalLeaderboard(); }
    catch (_) {
      res.setHeader("Content-Type", "application/json");
      res.setHeader("Cache-Control", "no-store");
      res.statusCode = 503;
      res.end(JSON.stringify({ error: "Leaderboard unavailable. Try again later." }));
      return;
    }
    res.setHeader("Content-Type", "application/json");
    res.setHeader("Cache-Control", "public, s-maxage=3, stale-while-revalidate=10");
    res.statusCode = 200;
    res.end(JSON.stringify(scores));
    return;
  }

  if (req.method === "POST") {
    let body = req.body;
    if (typeof body === "string") {
      try {
        body = JSON.parse(body);
      } catch (_) {
        body = {};
      }
    }
    const name = body?.display_name || body?.name;
    const score = body?.high_score ?? body?.score;

    try {
      const updatedScores = await submitGlobalScore(name, score);
      res.setHeader("Content-Type", "application/json");
      res.statusCode = 200;
      res.end(JSON.stringify({ success: true, scores: updatedScores }));
    } catch (err) {
      res.setHeader("Content-Type", "application/json");
      res.statusCode = 400;
      res.end(JSON.stringify({ error: err.message || "Invalid score submission" }));
    }
    return;
  }

  res.statusCode = 405;
  res.end("Method Not Allowed");
}
