import { invoke } from "./ipc";

export interface UpdateStatus {
  currentVersion: string;
  latestVersion: string;
  updateAvailable: boolean;
  notes: string;
  downloadUrl: string;
}

export const checkUpdate = (): Promise<UpdateStatus> => invoke("check_update");
