<script>
  import "../app.css";
  import { onMount } from 'svelte';
  import { check } from '@tauri-apps/plugin-updater';
  import { ask } from '@tauri-apps/plugin-dialog';
  import { relaunch } from '@tauri-apps/plugin-process';

  const LAST_UPDATE_CHECK_KEY = 'last_update_check';
  const CHECK_INTERVAL_MS = 24 * 60 * 60 * 1000; // 24 hours

  async function checkForUpdatesOnLaunch() {
    try {
      // Check if we should run update check (throttle to once per 24h)
      const lastCheckStr = localStorage.getItem(LAST_UPDATE_CHECK_KEY);
      const lastCheck = lastCheckStr ? parseInt(lastCheckStr) : 0;
      const now = Date.now();

      if (now - lastCheck < CHECK_INTERVAL_MS) {
        console.log('Update check skipped - checked recently');
        return;
      }

      console.log('Checking for updates on launch...');
      const update = await check();

      // Save check timestamp
      localStorage.setItem(LAST_UPDATE_CHECK_KEY, now.toString());

      if (update?.available) {
        console.log('Update available:', update.version);

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
          console.log('Downloading update...');
          await update.downloadAndInstall();

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
          }
        }
      } else {
        console.log('No updates available');
      }
    } catch (error) {
      console.error('Silent update check failed:', error);
      // Fail silently - don't bother the user
    }
  }

  onMount(() => {
    // Check for updates 5 seconds after launch (give app time to initialize)
    setTimeout(checkForUpdatesOnLaunch, 5000);
  });
</script>

<slot />
