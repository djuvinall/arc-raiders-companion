import { checkAppUpdate, checkDataUpdate, installAppUpdate, restartApp } from "./api";

export const updates = $state({
  app: null as string | null,
  data: null as string | null,
  checking: false,
  checked: false,
  installing: false,
  error: "",
});

// A failed check says nothing at launch: being offline isn't a problem worth a banner.
export async function checkForUpdates(quiet = true) {
  updates.checking = true;
  updates.error = "";
  const [app, data] = await Promise.allSettled([checkAppUpdate(), checkDataUpdate()]);
  updates.app = app.status === "fulfilled" ? app.value : null;
  updates.data = data.status === "fulfilled" ? data.value : null;
  if (!quiet && app.status === "rejected") updates.error = `Couldn't check for updates: ${app.reason}`;
  updates.checking = false;
  updates.checked = true;
}

export async function installUpdate() {
  updates.installing = true;
  updates.error = "";
  try {
    await installAppUpdate();
  } catch (e) {
    updates.error = `The update failed: ${e}`;
    updates.installing = false;
  }
}

export const applyData = () => restartApp();
