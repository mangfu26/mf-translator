/// 把 unix 秒级时间戳格式化为相对时间（超过一天显示绝对日期）。
export function formatRelativeTime(unixSeconds: number, now = Date.now()): string {
  const ms = unixSeconds * 1000;
  const diffSeconds = Math.floor((now - ms) / 1000);
  if (diffSeconds < 60) return "刚刚";
  if (diffSeconds < 3600) return `${Math.floor(diffSeconds / 60)} 分钟前`;
  if (diffSeconds < 86400) return `${Math.floor(diffSeconds / 3600)} 小时前`;

  const date = new Date(ms);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
}
