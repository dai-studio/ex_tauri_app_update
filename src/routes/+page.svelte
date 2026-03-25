<script lang="ts">
  import { invoke, Channel } from "@tauri-apps/api/core";

  type UpdateInfo = {
    version: string;
    currentVersion: string;
    notes: string | null;
  };

  type DownloadEvent =
    | { event: "started"; data: { contentLength: number | null } }
    | { event: "progress"; data: { chunkLength: number; downloaded: number } }
    | { event: "finished"; data: Record<string, never> };

  type Status =
    | { kind: "idle" }
    | { kind: "checking" }
    | { kind: "up-to-date" }
    | { kind: "available"; info: UpdateInfo }
    | { kind: "downloading"; downloaded: number; total: number | null }
    | { kind: "installing" }
    | { kind: "error"; message: string };

  let status = $state<Status>({ kind: "idle" });

  async function checkForUpdate() {
    status = { kind: "checking" };
    try {
      const info = await invoke<UpdateInfo | null>("check_update");
      if (info) {
        status = { kind: "available", info };
      } else {
        status = { kind: "up-to-date" };
      }
    } catch (e) {
      status = { kind: "error", message: String(e) };
    }
  }

  async function installUpdate() {
    status = { kind: "downloading", downloaded: 0, total: null };

    const channel = new Channel<DownloadEvent>();
    channel.onmessage = (event) => {
      if (event.event === "started") {
        status = {
          kind: "downloading",
          downloaded: 0,
          total: event.data.contentLength,
        };
      } else if (event.event === "progress") {
        if (status.kind === "downloading") {
          status = {
            kind: "downloading",
            downloaded: event.data.downloaded,
            total: status.total,
          };
        }
      } else if (event.event === "finished") {
        status = { kind: "installing" };
      }
    };

    try {
      await invoke("apply_update", { onEvent: channel });
    } catch (e) {
      status = { kind: "error", message: String(e) };
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function progressPercent(downloaded: number, total: number | null): number {
    if (!total) return 0;
    return Math.min(100, Math.round((downloaded / total) * 100));
  }
</script>

<main class="container">
  <h1>blank</h1>
  <p class="version">
    {#if status.kind === "available"}
      v{status.info.currentVersion}
    {:else}
      v0.1.0
    {/if}
  </p>

  <div class="update-card">
    {#if status.kind === "idle" || status.kind === "error"}
      <button onclick={checkForUpdate} class="btn-primary">
        Check for Updates
      </button>
      {#if status.kind === "error"}
        <p class="error">{status.message}</p>
      {/if}

    {:else if status.kind === "checking"}
      <div class="spinner"></div>
      <p class="status-text">Checking for updates...</p>

    {:else if status.kind === "up-to-date"}
      <p class="status-icon">✓</p>
      <p class="status-text">You're up to date.</p>
      <button onclick={checkForUpdate} class="btn-secondary">
        Check Again
      </button>

    {:else if status.kind === "available"}
      <p class="update-available">Update available: v{status.info.version}</p>
      {#if status.info.notes}
        <pre class="release-notes">{status.info.notes}</pre>
      {/if}
      <button onclick={installUpdate} class="btn-primary">
        Download & Install
      </button>

    {:else if status.kind === "downloading"}
      <p class="status-text">Downloading update...</p>
      <div class="progress-bar-track">
        <div
          class="progress-bar-fill"
          style="width: {progressPercent(status.downloaded, status.total)}%"
        ></div>
      </div>
      <p class="progress-label">
        {#if status.total}
          {formatBytes(status.downloaded)} / {formatBytes(status.total)}
          ({progressPercent(status.downloaded, status.total)}%)
        {:else}
          {formatBytes(status.downloaded)} downloaded
        {/if}
      </p>

    {:else if status.kind === "installing"}
      <div class="spinner"></div>
      <p class="status-text">Installing... the app will restart shortly.</p>
    {/if}
  </div>
</main>

<style>
  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;
    color: #0f0f0f;
    background-color: #f6f6f6;
    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #1a1a1a;
    }

    .update-card {
      background-color: #2a2a2a !important;
      border-color: #3a3a3a !important;
    }

    .release-notes {
      background-color: #1a1a1a !important;
      border-color: #3a3a3a !important;
      color: #ccc !important;
    }

    .btn-secondary {
      background-color: #2a2a2a !important;
      color: #f6f6f6 !important;
      border-color: #555 !important;
    }

    .progress-bar-track {
      background-color: #3a3a3a !important;
    }
  }

  .container {
    max-width: 480px;
    margin: 0 auto;
    padding: 10vh 1.5rem 2rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  h1 {
    font-size: 2.5rem;
    font-weight: 700;
    margin-bottom: 0.25rem;
    letter-spacing: -0.03em;
  }

  .version {
    font-size: 0.9rem;
    color: #888;
    margin-bottom: 2.5rem;
  }

  .update-card {
    width: 100%;
    background-color: #fff;
    border: 1px solid #e5e5e5;
    border-radius: 12px;
    padding: 2rem 1.5rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  }

  .btn-primary {
    padding: 0.65em 1.8em;
    font-size: 1rem;
    font-weight: 600;
    font-family: inherit;
    color: #fff;
    background-color: #396cd8;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: background-color 0.2s, transform 0.1s;
  }

  .btn-primary:hover {
    background-color: #2d5bbf;
  }

  .btn-primary:active {
    transform: scale(0.98);
  }

  .btn-secondary {
    padding: 0.55em 1.4em;
    font-size: 0.9rem;
    font-weight: 500;
    font-family: inherit;
    color: #444;
    background-color: #f0f0f0;
    border: 1px solid #ddd;
    border-radius: 8px;
    cursor: pointer;
    transition: border-color 0.2s;
  }

  .btn-secondary:hover {
    border-color: #396cd8;
  }

  .status-text {
    margin: 0;
    color: #666;
    font-size: 0.95rem;
  }

  .status-icon {
    font-size: 2rem;
    margin: 0;
    color: #22c55e;
  }

  .update-available {
    font-weight: 600;
    font-size: 1.05rem;
    margin: 0;
    color: #396cd8;
  }

  .release-notes {
    width: 100%;
    margin: 0;
    padding: 0.75rem 1rem;
    font-family: inherit;
    font-size: 0.85rem;
    line-height: 1.6;
    color: #555;
    background-color: #f8f8f8;
    border: 1px solid #e5e5e5;
    border-radius: 8px;
    text-align: left;
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 180px;
    overflow-y: auto;
  }

  .error {
    margin: 0;
    font-size: 0.85rem;
    color: #dc2626;
    word-break: break-word;
  }

  .progress-bar-track {
    width: 100%;
    height: 8px;
    background-color: #e5e5e5;
    border-radius: 999px;
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background-color: #396cd8;
    border-radius: 999px;
    transition: width 0.2s ease;
  }

  .progress-label {
    margin: 0;
    font-size: 0.82rem;
    color: #888;
  }

  /* Spinner */
  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid #e5e5e5;
    border-top-color: #396cd8;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
