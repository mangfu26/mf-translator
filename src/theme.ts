export type ThemePref = "light" | "dark" | "system";

const STORAGE_KEY = "mf.theme";

export function loadThemePref(): ThemePref {
  const value = localStorage.getItem(STORAGE_KEY);
  return value === "light" || value === "dark" || value === "system" ? value : "system";
}

export function saveThemePref(pref: ThemePref): void {
  localStorage.setItem(STORAGE_KEY, pref);
}

export function resolveEffectiveTheme(pref: ThemePref, prefersDark: boolean): "light" | "dark" {
  if (pref === "system") return prefersDark ? "dark" : "light";
  return pref;
}

function prefersDarkScheme(): boolean {
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

export function applyTheme(pref: ThemePref): void {
  const effective = resolveEffectiveTheme(pref, prefersDarkScheme());
  document.documentElement.classList.toggle("dark", effective === "dark");
}

export function watchSystemTheme(onChange: () => void): void {
  window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", onChange);
}
