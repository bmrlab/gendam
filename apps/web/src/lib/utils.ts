export { cn, twx } from '@muse/tailwind/utils'

export function formatDuration(seconds: number) {
  let d = seconds < 0 ? 0 : seconds
  var h = Math.floor(d / 3600)
  var m = Math.floor((d % 3600) / 60)
  var s = Math.floor((d % 3600) % 60)
  var hDisplay = h > 0 ? (h < 10 ? '0' + h : h) + ':' : '00:'
  var mDisplay = m > 0 ? (m < 10 ? '0' + m : m) + ':' : '00:'
  var sDisplay = s > 0 ? (s < 10 ? '0' + s : s) : '00'
  return hDisplay + mDisplay + sDisplay
}

export function formatBytes(bytes: number, decimals: number = 2) {
  bytes = bytes < 0 ? 0 : bytes
  decimals = decimals < 0 ? 0 : decimals
  if (bytes === 0) {
    return '0 Bytes'
  }
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(decimals)) + ' ' + sizes[i]
}

export function formatDateTime(date: string) {
  // format datetime as 'YYYY-MM-DD hh::mm::ss', padding with 0
  const pad = (n: number): string => (n < 10 ? '0' : '') + n
  const d = new Date(date)
  const year = d.getFullYear()
  const month = d.getMonth() + 1
  const day = d.getDate()
  const hour = d.getHours()
  const minute = d.getMinutes()
  const second = d.getSeconds()
  return `${year}-${pad(month)}-${pad(day)} ${pad(hour)}:${pad(minute)}:${pad(second)}`
}
