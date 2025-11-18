<script lang="ts">
  import { onMount } from 'svelte';
  import { check } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { ask } from '@tauri-apps/plugin-dialog';

  let checkingForUpdate = $state(false);
  let updateAvailable = $state(false);
  let updateInfo = $state<any>(null);
  let downloading = $state(false);
  let downloadProgress = $state(0);
  let downloadSize = $state({ current: 0, total: 0 });
  let installing = $state(false);
  let updateError = $state("");
  let currentVersion = $state("0.1.0");

  async function checkForUpdates(silent: boolean = false) {
    if (checkingForUpdate) return;

    checkingForUpdate = true;
    updateError = "";

    try {
      console.log('Checking for updates...');
      const update = await check();

      if (update?.available) {
        console.log('Update available:', update.version);
        updateAvailable = true;
        updateInfo = update;

        if (!silent) {
          // Show update notification
          const yes = await ask(
            `Une nouvelle version ${update.version} est disponible. Voulez-vous la télécharger maintenant?`,
            {
              title: 'Mise à jour disponible',
              kind: 'info',
              okLabel: 'Oui',
              cancelLabel: 'Plus tard'
            }
          );

          if (yes) {
            await downloadAndInstall();
          }
        }
      } else {
        console.log('No updates available');
        updateAvailable = false;
        if (!silent) {
          updateError = "Vous utilisez la dernière version";
        }
      }
    } catch (error) {
      console.error('Update check failed:', error);
      updateError = `Erreur lors de la vérification: ${error}`;
    } finally {
      checkingForUpdate = false;
    }
  }

  async function downloadAndInstall() {
    if (!updateInfo) return;

    downloading = true;
    downloadProgress = 0;
    updateError = "";

    try {
      console.log('Downloading update...');

      // Download with progress tracking
      await updateInfo.downloadAndInstall((event: any) => {
        switch (event.event) {
          case 'Started':
            downloadSize.total = event.data.contentLength || 0;
            console.log('Download started, size:', downloadSize.total);
            break;
          case 'Progress':
            downloadSize.current = event.data.chunkLength;
            if (downloadSize.total > 0) {
              downloadProgress = Math.round((downloadSize.current / downloadSize.total) * 100);
            }
            console.log(`Download progress: ${downloadProgress}%`);
            break;
          case 'Finished':
            console.log('Download finished');
            downloading = false;
            installing = true;
            break;
        }
      });

      console.log('Update installed successfully');

      // Ask to restart
      const restart = await ask(
        'La mise à jour a été installée. Voulez-vous redémarrer maintenant?',
        {
          title: 'Mise à jour installée',
          kind: 'info',
          okLabel: 'Redémarrer',
          cancelLabel: 'Plus tard'
        }
      );

      if (restart) {
        await relaunch();
      } else {
        installing = false;
        updateError = "Redémarrez l'application pour appliquer la mise à jour";
      }
    } catch (error) {
      console.error('Update download/install failed:', error);
      updateError = `Erreur lors de l'installation: ${error}`;
      downloading = false;
      installing = false;
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  }

  onMount(() => {
    // Check for updates on mount (silent)
    checkForUpdates(true);
  });
</script>

<div class="update-checker">
  <div class="update-section">
    <h3>Version actuelle</h3>
    <div class="version-badge">{currentVersion}</div>
  </div>

  <div class="update-section">
    <h3>Mises à jour</h3>

    {#if checkingForUpdate}
      <div class="status-card checking">
        <div class="spinner"></div>
        <p>Vérification des mises à jour...</p>
      </div>
    {:else if downloading}
      <div class="status-card downloading">
        <p>Téléchargement de la mise à jour...</p>
        <div class="progress-bar">
          <div class="progress-fill" style="width: {downloadProgress}%"></div>
        </div>
        <p class="progress-text">
          {downloadProgress}% - {formatBytes(downloadSize.current)} / {formatBytes(downloadSize.total)}
        </p>
      </div>
    {:else if installing}
      <div class="status-card installing">
        <div class="spinner"></div>
        <p>Installation de la mise à jour...</p>
      </div>
    {:else if updateAvailable && updateInfo}
      <div class="status-card update-available">
        <div class="update-icon">🎉</div>
        <h4>Nouvelle version disponible!</h4>
        <p class="update-version">Version {updateInfo.version}</p>
        <button class="download-button" onclick={downloadAndInstall}>
          Télécharger et installer
        </button>
      </div>
    {:else}
      <div class="status-card up-to-date">
        <div class="check-icon">✓</div>
        <p>Vous utilisez la dernière version</p>
      </div>
    {/if}

    {#if updateError}
      <div class="error-message">{updateError}</div>
    {/if}

    <button
      class="check-button"
      onclick={() => checkForUpdates(false)}
      disabled={checkingForUpdate || downloading || installing}
    >
      Vérifier les mises à jour
    </button>
  </div>
</div>

<style>
  .update-checker {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .update-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .update-section h3 {
    font-size: 14px;
    font-weight: 600;
    color: #BDBDBD;
    margin: 0;
  }

  .version-badge {
    display: inline-flex;
    padding: 8px 16px;
    background: #202020;
    border: 1px solid #333333;
    border-radius: 6px;
    font-size: 16px;
    font-weight: 600;
    color: #4FB094;
    width: fit-content;
  }

  .status-card {
    padding: 20px;
    background: #202020;
    border: 1px solid #333333;
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    text-align: center;
  }

  .status-card.checking,
  .status-card.downloading,
  .status-card.installing {
    border-color: #4FB094;
  }

  .status-card.update-available {
    border-color: #4FB094;
    background: #1a3d32;
  }

  .status-card.up-to-date {
    border-color: #4FB094;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 3px solid #333333;
    border-top-color: #4FB094;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .update-icon,
  .check-icon {
    font-size: 32px;
  }

  .update-version {
    font-size: 18px;
    font-weight: 600;
    color: #4FB094;
    margin: 0;
  }

  .status-card h4 {
    font-size: 16px;
    font-weight: 600;
    color: #BDBDBD;
    margin: 0;
  }

  .status-card p {
    font-size: 14px;
    color: #8E8E93;
    margin: 0;
  }

  .progress-bar {
    width: 100%;
    height: 8px;
    background: #333333;
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #4FB094, #3d9178);
    transition: width 0.3s ease;
  }

  .progress-text {
    font-size: 12px;
    color: #BDBDBD;
  }

  .download-button,
  .check-button {
    padding: 10px 20px;
    background: #4FB094;
    color: #141414;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .download-button:hover,
  .check-button:hover {
    background: #3d9178;
    transform: translateY(-1px);
  }

  .download-button:active,
  .check-button:active {
    transform: translateY(0);
  }

  .check-button {
    margin-top: 8px;
    width: 100%;
  }

  .check-button:disabled {
    background: #333333;
    color: #666666;
    cursor: not-allowed;
    transform: none;
  }

  .error-message {
    padding: 12px;
    background: rgba(255, 59, 48, 0.1);
    border: 1px solid rgba(255, 59, 48, 0.3);
    border-radius: 6px;
    color: #ff3b30;
    font-size: 13px;
    text-align: center;
  }
</style>
