export const fetch = async (url: string, options?: unknown) => {
  console.log("[Mock Tauri HTTP] fetch:", url, options);

  if (url.includes("erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game=")) {
    const id = url.split("game=")[1].split("&")[0];
    try {
      const res = await window.fetch(`./demo-data/${id}.html`);
      if (res.ok) {
        const html = await res.text();
        return {
          ok: true,
          text: async () => html,
          json: async () => ({}),
          arrayBuffer: async () => new TextEncoder().encode(html).buffer
        };
      }
    } catch (e) {
      console.error("Failed to fetch demo html", e);
    }
  }

  // Erogamescape mock
  if (url.includes("erogamescape")) {
    return {
      ok: true,
      text: async () => "1\n2\n3", // dummy csv format if expected
      json: async () => ({})
    };
  }

  // VNDB mock
  if (url.includes("vndb")) {
    return {
      ok: true,
      json: async () => ({ results: [] }),
      text: async () => "{}"
    };
  }

  // fallback
  return {
    ok: true,
    text: async () => "",
    json: async () => ({}),
    arrayBuffer: async () => new ArrayBuffer(0)
  };
};
